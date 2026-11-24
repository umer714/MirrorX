package expo.modules.mirrorxcapture

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.pm.ServiceInfo
import android.content.Intent
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper

class MirrorXProjectionService : Service() {
  private var projection: MediaProjection? = null

  override fun onCreate() {
    super.onCreate()
    createNotificationChannel()
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    if (intent?.action == ACTION_STOP) {
      stopProjection()
      stopSelf()
      return START_NOT_STICKY
    }

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
      projection = manager.getMediaProjection(resultCode, resultData)
      projection?.registerCallback(object : MediaProjection.Callback() {
        override fun onStop() {
          projection = null
          stopForeground(STOP_FOREGROUND_REMOVE)
          stopSelf()
        }
      }, Handler(Looper.getMainLooper()))
    } catch (_: Throwable) {
      stopProjection()
      stopSelf()
      return START_NOT_STICKY
    }
    return START_NOT_STICKY
  }

  private fun stopProjection() {
    projection?.stop()
    projection = null
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
      .setContentText("Screen capture is active on this device. Tap Stop in MirrorX to end it.")
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
    private const val CHANNEL_ID = "mirrorx_screen_capture"
    private const val NOTIFICATION_ID = 7402
  }
}
