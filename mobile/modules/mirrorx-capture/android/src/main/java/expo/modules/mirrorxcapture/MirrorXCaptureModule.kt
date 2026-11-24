package expo.modules.mirrorxcapture

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Build
import android.media.projection.MediaProjectionManager
import expo.modules.kotlin.Promise
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

private const val SCREEN_CAPTURE_REQUEST_CODE = 7401

class MirrorXCaptureModule : Module() {
  private var pendingPromise: Promise? = null
  private var approvedResultCode: Int? = null
  private var approvedResultData: Intent? = null

  override fun definition() = ModuleDefinition {
    Name("MirrorXCapture")

    AsyncFunction("requestScreenCapturePermission") { promise: Promise ->
      if (pendingPromise != null) {
        promise.reject("ERR_CAPTURE_REQUEST_IN_PROGRESS", "A screen-capture permission request is already in progress.", null)
        return@AsyncFunction
      }

      val activity = appContext.throwingActivity
      val manager = activity.getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
      pendingPromise = promise

      try {
        activity.startActivityForResult(
          manager.createScreenCaptureIntent(),
          SCREEN_CAPTURE_REQUEST_CODE
        )
      } catch (error: Throwable) {
        pendingPromise = null
        promise.reject("ERR_CAPTURE_PERMISSION_LAUNCH", error.message ?: "Unable to open Android screen-capture consent.", error)
      }
    }

    Function("hasApprovedScreenCapturePermission") {
      approvedResultCode == Activity.RESULT_OK && approvedResultData != null
    }

    Function("clearApprovedScreenCapturePermission") {
      approvedResultCode = null
      approvedResultData = null
      true
    }

    Function("stopProjectionSession") {
      val context = appContext.reactContext ?: return@Function false
      val stopIntent = Intent(context, MirrorXProjectionService::class.java).apply {
        action = MirrorXProjectionService.ACTION_STOP
      }
      context.startService(stopIntent)
      approvedResultCode = null
      approvedResultData = null
      true
    }

    Function("getCaptureStats") {
      val context = appContext.reactContext ?: return@Function mapOf(
        "active" to false, "frames" to 0L, "encodedBytes" to 0L, "keyFrames" to 0L, "codecConfigBytes" to 0, "width" to 0, "height" to 0
      )
      val prefs = context.getSharedPreferences(MirrorXProjectionService.PREFS, Context.MODE_PRIVATE)
      mapOf(
        "active" to prefs.getBoolean(MirrorXProjectionService.KEY_ACTIVE, false),
        "frames" to prefs.getLong(MirrorXProjectionService.KEY_FRAMES, 0L),
        "encodedBytes" to prefs.getLong(MirrorXProjectionService.KEY_ENCODED_BYTES, 0L),
        "keyFrames" to prefs.getLong(MirrorXProjectionService.KEY_KEY_FRAMES, 0L),
        "codecConfigBytes" to prefs.getInt(MirrorXProjectionService.KEY_CODEC_CONFIG_BYTES, 0),
        "width" to prefs.getInt(MirrorXProjectionService.KEY_WIDTH, 0),
        "height" to prefs.getInt(MirrorXProjectionService.KEY_HEIGHT, 0)
      )
    }

    OnActivityResult { _, payload ->
      if (payload.requestCode != SCREEN_CAPTURE_REQUEST_CODE) {
        return@OnActivityResult
      }

      val granted = payload.resultCode == Activity.RESULT_OK && payload.data != null
      if (granted) {
        approvedResultCode = payload.resultCode
        approvedResultData = payload.data
        try {
          val context = appContext.reactContext
            ?: throw IllegalStateException("Android application context is unavailable.")
          val serviceIntent = Intent(context, MirrorXProjectionService::class.java).apply {
            putExtra(MirrorXProjectionService.EXTRA_RESULT_CODE, payload.resultCode)
            putExtra(MirrorXProjectionService.EXTRA_RESULT_DATA, payload.data)
          }
          if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            context.startForegroundService(serviceIntent)
          } else {
            context.startService(serviceIntent)
          }
        } catch (error: Throwable) {
          approvedResultCode = null
          approvedResultData = null
          pendingPromise?.reject(
            "ERR_PROJECTION_SERVICE_START",
            error.message ?: "Unable to start the visible screen-session service.",
            error
          )
          pendingPromise = null
          return@OnActivityResult
        }
      } else {
        approvedResultCode = null
        approvedResultData = null
      }

      pendingPromise?.resolve(
        mapOf(
          "granted" to granted,
          "status" to if (granted) "approved" else "denied"
        )
      )
      pendingPromise = null
    }
  }
}
