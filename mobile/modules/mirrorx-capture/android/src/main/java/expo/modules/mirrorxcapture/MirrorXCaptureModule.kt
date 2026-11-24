package expo.modules.mirrorxcapture

import android.app.Activity
import android.content.Context
import android.content.Intent
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

    OnActivityResult { _, payload ->
      if (payload.requestCode != SCREEN_CAPTURE_REQUEST_CODE) {
        return@OnActivityResult
      }

      val granted = payload.resultCode == Activity.RESULT_OK && payload.data != null
      if (granted) {
        approvedResultCode = payload.resultCode
        approvedResultData = payload.data
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
