package expo.modules.mirrorxcapture

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import kotlin.math.min

class MirrorXProjectionService : Service() {
  private var projection: MediaProjection? = null
  private var virtualDisplay: VirtualDisplay? = null
  private var encoder: MediaCodec? = null
  @Volatile private var draining = false
  private var drainThread: Thread? = null
  @Volatile private var encodedFrames = 0L
  @Volatile private var encodedBytes = 0L
  @Volatile private var keyFrames = 0L
  private var codecConfigBytes = 0
  private var captureWidth = 0
  private var captureHeight = 0

  override fun onCreate() {
    super.onCreate()
    createNotificationChannel()
    saveStats(active = false, frames = 0L, bytes = 0L, keyFrames = 0L, configBytes = 0, width = 0, height = 0)
    saveLastError("")
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    if (intent?.action == ACTION_STOP) {
      stopProjection()
      stopSelf()
      return START_NOT_STICKY
    }
    if (projection != null) return START_NOT_STICKY

    val resultCode = intent?.getIntExtra(EXTRA_RESULT_CODE, 0) ?: 0
    val resultData = if (Build.VERSION.SDK_INT >= 33) {
      intent?.getParcelableExtra(EXTRA_RESULT_DATA, Intent::class.java)
    } else {
      @Suppress("DEPRECATION")
      intent?.getParcelableExtra(EXTRA_RESULT_DATA)
    }
    if (resultCode == 0 || resultData == null) {
      stopSelf()
      return START_NOT_STICKY
    }

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
      startForeground(NOTIFICATION_ID, buildNotification(), ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PROJECTION)
    } else {
      startForeground(NOTIFICATION_ID, buildNotification())
    }

    saveLastError("")
    try {
      val manager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
      val activeProjection = manager.getMediaProjection(resultCode, resultData)
        ?: throw IllegalStateException("Android did not provide an active screen projection.")
      projection = activeProjection
      activeProjection.registerCallback(object : MediaProjection.Callback() {
        override fun onStop() {
          releaseCaptureResources()
          projection = null
          saveStats(active = false, frames = encodedFrames, bytes = encodedBytes, keyFrames = keyFrames, configBytes = codecConfigBytes, width = 0, height = 0)
          stopForeground(STOP_FOREGROUND_REMOVE)
          stopSelf()
        }
      }, Handler(Looper.getMainLooper()))

      val display = resources.displayMetrics
      val scale = min(1f, min(1280f / display.widthPixels, 720f / display.heightPixels))
      captureWidth = ((display.widthPixels * scale).toInt() / 2) * 2
      captureHeight = ((display.heightPixels * scale).toInt() / 2) * 2
      captureWidth = captureWidth.coerceAtLeast(2)
      captureHeight = captureHeight.coerceAtLeast(2)
      encodedFrames = 0L
      encodedBytes = 0L
      keyFrames = 0L
      codecConfigBytes = 0

      val format = MediaFormat.createVideoFormat(MIME_TYPE, captureWidth, captureHeight).apply {
        setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface)
        setInteger(MediaFormat.KEY_BIT_RATE, (captureWidth * captureHeight * 4).coerceIn(1_000_000, 8_000_000))
        setInteger(MediaFormat.KEY_FRAME_RATE, TARGET_FPS)
        setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 2)
      }
      val videoEncoder = MediaCodec.createEncoderByType(MIME_TYPE)
      encoder = videoEncoder
      videoEncoder.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
      val inputSurface = videoEncoder.createInputSurface()
      videoEncoder.start()

      virtualDisplay = activeProjection.createVirtualDisplay(
        "MirrorX-H264-Capture",
        captureWidth,
        captureHeight,
        display.densityDpi,
        DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
        inputSurface,
        null,
        Handler(Looper.getMainLooper())
      ) ?: throw IllegalStateException("Android could not create the screen capture display.")

      startEncoderDrain(videoEncoder)
      saveStats(active = true, frames = 0L, bytes = 0L, keyFrames = 0L, configBytes = 0, width = captureWidth, height = captureHeight)
    } catch (error: Throwable) {
      saveLastError(classifyCaptureError(error))
      stopProjection()
      saveLastError(classifyCaptureError(error))
      stopSelf()
      return START_NOT_STICKY
    }
    return START_NOT_STICKY
  }

  private fun startEncoderDrain(codec: MediaCodec) {
    draining = true
    drainThread = Thread({
      val info = MediaCodec.BufferInfo()
      while (draining) {
        try {
          when (val index = codec.dequeueOutputBuffer(info, 10_000)) {
            MediaCodec.INFO_TRY_AGAIN_LATER -> Unit
            MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
              val outputFormat = codec.outputFormat
              codecConfigBytes = listOf("csd-0", "csd-1", "csd-2")
                .sumOf { key -> if (outputFormat.containsKey(key)) outputFormat.getByteBuffer(key)?.remaining() ?: 0 else 0 }
              saveStats(true, encodedFrames, encodedBytes, keyFrames, codecConfigBytes, captureWidth, captureHeight)
            }
            else -> if (index >= 0) {
              try {
                if (info.size > 0) {
                  val output = codec.getOutputBuffer(index)
                  if (output != null && (info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) == 0) {
                    // This stage records encoder output only. Avoid allocating/copying a frame
                    // that is not yet consumed by a transport; the codec buffer is released below.
                    val bufferEnd = info.offset.toLong() + info.size.toLong()
                    if (info.offset >= 0 && info.size > 0 && bufferEnd <= output.capacity().toLong()) {
                      encodedFrames += 1
                      encodedBytes += info.size.toLong()
                      if ((info.flags and MediaCodec.BUFFER_FLAG_KEY_FRAME) != 0) keyFrames += 1
                      if (encodedFrames % 15L == 0L) {
                        saveStats(true, encodedFrames, encodedBytes, keyFrames, codecConfigBytes, captureWidth, captureHeight)
                      }
                    }
                  }
                }
              } finally {
                codec.releaseOutputBuffer(index, false)
              }
            }
          }
        } catch (_: IllegalStateException) {
          break
        } catch (_: IndexOutOfBoundsException) {
          // A codec may invalidate a buffer during shutdown; skip that output and continue draining.
        }
      }
    }, "MirrorX-H264-Drain").also { it.start() }
  }

  private fun saveLastError(code: String) {
    getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY_LAST_ERROR, code).apply()
  }

  private fun classifyCaptureError(error: Throwable): String = when (error) {
    is SecurityException -> "CAPTURE_PERMISSION_OR_FOREGROUND_SERVICE"
    is IllegalArgumentException -> "INVALID_CAPTURE_CONFIGURATION"
    is IllegalStateException -> "CAPTURE_CODEC_OR_PROJECTION_STATE"
    else -> "CAPTURE_START_FAILED"
  }

  private fun saveStats(active: Boolean, frames: Long, bytes: Long, keyFrames: Long, configBytes: Int, width: Int, height: Int) {
    getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
      .putBoolean(KEY_ACTIVE, active)
      .putLong(KEY_FRAMES, frames)
      .putLong(KEY_ENCODED_BYTES, bytes)
      .putLong(KEY_KEY_FRAMES, keyFrames)
      .putInt(KEY_CODEC_CONFIG_BYTES, configBytes)
      .putInt(KEY_WIDTH, width)
      .putInt(KEY_HEIGHT, height)
      .apply()
  }

  private fun releaseCaptureResources() {
    virtualDisplay?.release()
    virtualDisplay = null
    draining = false
    drainThread?.join(300)
    drainThread = null
    encoder?.let { codec ->
      try { codec.stop() } catch (_: IllegalStateException) { }
      try { codec.release() } catch (_: IllegalStateException) { }
    }
    encoder = null
  }

  private fun stopProjection() {
    releaseCaptureResources()
    val active = projection
    projection = null
    active?.stop()
    saveStats(false, encodedFrames, encodedBytes, keyFrames, codecConfigBytes, 0, 0)
    stopForeground(STOP_FOREGROUND_REMOVE)
  }

  private fun createNotificationChannel() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val channel = NotificationChannel(CHANNEL_ID, "MirrorX screen sharing", NotificationManager.IMPORTANCE_LOW)
      getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
    }
  }

  private fun buildNotification(): Notification {
    val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) Notification.Builder(this, CHANNEL_ID)
    else {
      @Suppress("DEPRECATION")
      Notification.Builder(this)
    }
    return builder.setSmallIcon(android.R.drawable.ic_menu_view)
      .setContentTitle("MirrorX screen session active")
      .setContentText("Screen video is being encoded locally. Tap Stop in MirrorX to end it.")
      .setOngoing(true)
      .build()
  }

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onDestroy() {
    stopProjection()
    super.onDestroy()
  }

  companion object {
    const val ACTION_STOP = "expo.modules.mirrorxcapture.STOP"
    const val EXTRA_RESULT_CODE = "mirrorx_result_code"
    const val EXTRA_RESULT_DATA = "mirrorx_result_data"
    const val PREFS = "mirrorx_capture_stats"
    const val KEY_ACTIVE = "active"
    const val KEY_FRAMES = "frames"
    const val KEY_ENCODED_BYTES = "encoded_bytes"
    const val KEY_KEY_FRAMES = "key_frames"
    const val KEY_CODEC_CONFIG_BYTES = "codec_config_bytes"
    const val KEY_WIDTH = "width"
    const val KEY_HEIGHT = "height"
    const val KEY_LAST_ERROR = "last_error"
    private const val MIME_TYPE = "video/avc"
    private const val TARGET_FPS = 30
    private const val CHANNEL_ID = "mirrorx_screen_capture"
    private const val NOTIFICATION_ID = 7402
  }
}
