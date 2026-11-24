package expo.modules.mirrorxcapture

internal data class MirrorXEncodedFrame(
  val data: ByteArray,
  val keyframe: Boolean,
  val ptsMicros: Long
)

/**
 * JNI access to the shared Rust H.264 queue.
 *
 * The native library is packaged for Android ABIs by the module's Gradle
 * pre-build task. Keep this wrapper internal to the capture module.
 */
internal object MirrorXNativeQueue {
  private var libraryLoaded = false

  @Synchronized
  fun ensureLoaded(): Boolean {
    if (libraryLoaded) return true
    return try {
      System.loadLibrary("mirrorx_media")
      System.loadLibrary("mirrorx_capture_jni")
      libraryLoaded = true
      true
    } catch (_: UnsatisfiedLinkError) {
      false
    } catch (_: SecurityException) {
      false
    }
  }

  @JvmStatic external fun nativeCreate(capacity: Int): Long
  @JvmStatic external fun nativePush(queue: Long, data: ByteArray, keyframe: Boolean, ptsMicros: Long): Int
  @JvmStatic external fun nativePop(queue: Long, maxBytes: Int): MirrorXEncodedFrame?
  @JvmStatic external fun nativeDestroy(queue: Long)
}
