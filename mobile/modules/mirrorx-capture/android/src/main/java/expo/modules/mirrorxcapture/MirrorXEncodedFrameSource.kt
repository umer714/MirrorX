package expo.modules.mirrorxcapture

/**
 * Single-consumer bridge from the bounded Rust H.264 queue to a native
 * transport adapter (for example, a WebRTC sender).
 *
 * This class deliberately does not create a network connection or transmit
 * frames. The owner must provide an authenticated, consent-approved consumer.
 * Only one instance may drain a queue handle at a time.
 */
internal class MirrorXEncodedFrameSource(
  private val queueHandle: Long,
  private val onFrame: (MirrorXEncodedFrame) -> Unit,
  private val onFailure: (Throwable) -> Unit = {}
) : AutoCloseable {
  @Volatile
  private var running = false

  private var worker: Thread? = null
  @Volatile private var generation = 0L

  @Synchronized
  fun start(): Boolean {
    if (running || queueHandle == 0L || !MirrorXNativeQueue.ensureLoaded()) return false

    running = true
    val workerGeneration = ++generation
    worker = Thread({
      try {
        while (running && generation == workerGeneration && !Thread.currentThread().isInterrupted) {
          val frame = MirrorXNativeQueue.nativePop(queueHandle, MAX_ACCESS_UNIT_BYTES)
          if (frame != null) {
            onFrame(frame)
          } else {
            // Avoid spinning at 100% CPU while the encoder has no output.
            try {
              Thread.sleep(EMPTY_QUEUE_SLEEP_MS)
            } catch (_: InterruptedException) {
              Thread.currentThread().interrupt()
              break
            }
          }
        }
      } catch (error: Throwable) {
        if (running) runCatching { onFailure(error) }
      } finally {
        if (generation == workerGeneration) {
          running = false
        }
      }
    }, "MirrorX-H264-Queue-Consumer").also {
      it.isDaemon = true
      it.start()
    }
    return true
  }

  @Synchronized
  override fun close() {
    running = false
    generation += 1
    val activeWorker = worker
    worker = null
    activeWorker?.interrupt()
    if (activeWorker != null && activeWorker !== Thread.currentThread()) {
      var interrupted = false
      // Do not return while JNI may still be draining the native queue. The
      // owner may destroy the queue handle immediately after close() returns.
      while (activeWorker.isAlive) {
        try {
          activeWorker.join()
        } catch (_: InterruptedException) {
          interrupted = true
        }
      }
      if (interrupted) Thread.currentThread().interrupt()
    }
  }

  fun isRunning(): Boolean = running

  private companion object {
    const val MAX_ACCESS_UNIT_BYTES = 16 * 1024 * 1024
    const val EMPTY_QUEUE_SLEEP_MS = 4L
  }
}
