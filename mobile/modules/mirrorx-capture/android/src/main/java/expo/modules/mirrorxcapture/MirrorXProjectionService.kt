package expo.modules.mirrorxcapture

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.graphics.PixelFormat
import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.media.ImageReader
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper

class MirrorXProjectionService : Service() {
  private var projection: MediaProjection? = null
  private var virtualDisplay: VirtualDisplay? = null
  private var imageReader: ImageReader? = null
  private var frameCount = 0L

  override fun onCreate() {
    super.onCreate()
    createNotificationChannel()
    saveStats(active = false, frames = 0L, width = 0, height = 0)
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
      startForeground(
        NOTIFICATION_ID,
        buildNotification(),
        ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PROJECTION
      )
    } else {
      startForeground(NOTIFICATION_ID, buildNotification())
    }

    try {
      val manager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
      val activeProjection = manager.getMediaProjection(resultCode, resultData)
      projection = activeProjection
      activeProjection.registerCallback(object : MediaProjection.Callback() {
        override fun onStop() {
          releaseCaptureResources()
          projection = null
          saveStats(active = false, frames = frameCount, width = 0, height = 0)
          stopForeground(STOP_FOREGROUND_REMOVE)
          stopSelf()
        }
      }, Handler(Looper.getMainLooper()))

      val metrics = resources.displayMetrics
      val width = metrics.widthPixels.coerceAtLeast(1)
      val height = metrics.heightPixels.coerceAtLeast(1)
      val density = metrics.densityDpi

      imageReader = ImageReader.newInstance(width, height, PixelFormat.RGBA_8888, 2)
      imageReader?.setOnImageAvailableListener({ reader ->
        var image = reader.acquireLatestImage()
        if (image != null) {
          frameCount += 1
          saveStats(active = true, frames = frameCount, width = width, height = height)
          image.close()
          image = null
        }
      }, Handler(Looper.getMainLooper()))

      virtualDisplay = activeProjection.createVirtualDisplay(
        "MirrorX-Capture",
        width,
        height,
        density,
        DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
        imageReader?.surface,
        null,
        Handler(Looper.getMainLooper())
      )

      if (virtualDisplay == null) {
        throw IllegalStateException("Android could not create the screen capture display.")
      }
      frameCount = 0L
      saveStats(active = true, frames = frameCount, width = width, height = height)
    } catch (error: Throwable) {
      stopProjection()
      stopSelf()
      return START_NOT_STICKY
    }
    return START_NOT_STICKY
  }

  private fun saveStats(active: Boolean, frames: Long, width: Int, height: Int) {
    getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
      .putBoolean(KEY_ACTIVE, active)
      .putLong(KEY_FRAMES, frames)
      .putInt(KEY_WIDTH, width)
      .putInt(KEY_HEIGHT, height)
      .apply()
  }

  private fun releaseCaptureResources() {
    virtualDisplay?.release()
    virtualDisplay = null
    imageReader?.setOnImageAvailableListener(null, null)
    imageReader?.close()
    imageReader = null
  }

  private fun stopProjection() {
    releaseCaptureResources()
    val active = projection
    projection = null
    active?.stop()
    saveStats(active = false, frames = frameCount, width = 0, height = 0)
    stopForeground(STOP_FOREGROUND_REMOVE)
  }

  private fun createNotificationChannel() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val channel = NotificationChannel(
        CHANNEL_ID,
        "MirrorX screen sharing",
        NotificationManager.IMPORTANCE_LOW
      )
      getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
    }
  }

  private fun buildNotification(): Notification {
    val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      Notification.Builder(this, CHANNEL_ID)
    } else {
      @Suppress("DEPRECATION")
      Notification.Builder(this)
    }
    return builder
      .setSmallIcon(android.R.drawable.ic_menu_view)
      .setContentTitle("MirrorX screen session active")
      .setContentText("Your screen is being captured locally. Tap Stop in MirrorX to end it.")
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
    const val KEY_WIDTH = "width"
    const val KEY_HEIGHT = "height"
    private const val CHANNEL_ID = "mirrorx_screen_capture"
    private const val NOTIFICATION_ID = 7402
  }
}
