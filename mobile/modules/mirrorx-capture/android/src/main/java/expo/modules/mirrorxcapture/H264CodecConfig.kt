package expo.modules.mirrorxcapture

import java.io.ByteArrayOutputStream

/**
 * Converts Android MediaCodec codec-specific data into Annex-B parameter sets.
 *
 * MediaCodec implementations expose CSD in more than one form: Annex-B
 * start-code-prefixed SPS/PPS, raw SPS/PPS NAL units, or an
 * AVCDecoderConfigurationRecord (AVCC). The encoder queue and desktop sender
 * use Annex-B consistently, so normalize before caching/prefixing IDR frames.
 */
internal object H264CodecConfig {
  private val startCode = byteArrayOf(0, 0, 0, 1)

  fun toAnnexB(parts: List<ByteArray>): ByteArray {
    val output = ByteArrayOutputStream()
    parts.forEach { part ->
      if (part.isEmpty()) return@forEach
      val normalized = when {
        isAnnexB(part) -> part
        isAvcConfiguration(part) -> parseAvcConfiguration(part)
        isParameterSet(part) -> startCode + part
        else -> throw IllegalArgumentException("Unsupported H.264 codec-specific data")
      }
      output.write(normalized)
    }
    return output.toByteArray()
  }

  private fun isAnnexB(data: ByteArray): Boolean =
    data.size >= 4 && data[0] == 0.toByte() && data[1] == 0.toByte() &&
      ((data[2] == 1.toByte()) || (data[2] == 0.toByte() && data[3] == 1.toByte()))

  private fun isAvcConfiguration(data: ByteArray): Boolean =
    data.size >= 7 && data[0] == 1.toByte() &&
      (data[4].toInt() and 0xfc) == 0xfc && (data[5].toInt() and 0xe0) == 0xe0

  private fun isParameterSet(data: ByteArray): Boolean {
    val type = data[0].toInt() and 0x1f
    return type == 7 || type == 8
  }

  private fun parseAvcConfiguration(data: ByteArray): ByteArray {
    require(data.size >= 7 && data[0] == 1.toByte()) { "Invalid AVC configuration record" }
    require((data[4].toInt() and 0xfc) == 0xfc && (data[5].toInt() and 0xe0) == 0xe0) {
      "Invalid AVC configuration reserved bits"
    }
    var cursor = 6
    val output = ByteArrayOutputStream()

    fun readParameterSet(expectedType: Int) {
      require(cursor + 2 <= data.size) { "Truncated AVC parameter-set length" }
      val length = ((data[cursor].toInt() and 0xff) shl 8) or (data[cursor + 1].toInt() and 0xff)
      cursor += 2
      require(length > 0 && cursor + length <= data.size) { "Invalid AVC parameter-set length" }
      val nal = data.copyOfRange(cursor, cursor + length)
      require((nal[0].toInt() and 0x1f) == expectedType) { "Unexpected AVC parameter-set type" }
      output.write(startCode)
      output.write(nal)
      cursor += length
    }

    val spsCount = data[5].toInt() and 0x1f
    require(spsCount > 0) { "AVC configuration is missing SPS" }
    repeat(spsCount) { readParameterSet(7) }

    require(cursor < data.size) { "AVC configuration is missing PPS count" }
    val ppsCount = data[cursor++].toInt() and 0xff
    require(ppsCount > 0) { "AVC configuration is missing PPS" }
    repeat(ppsCount) { readParameterSet(8) }

    // High-profile AVC records may carry optional SPS-extension fields.
    // Validate and skip them; the baseline SPS/PPS are what the receiver needs.
    if (cursor < data.size) {
      val profile = data[1].toInt() and 0xff
      require(profile in HIGH_PROFILES && cursor + 4 <= data.size) {
        "Unexpected trailing AVC configuration data"
      }
      require((data[cursor].toInt() and 0xfc) == 0xfc &&
        (data[cursor + 1].toInt() and 0xf8) == 0xf8 &&
        (data[cursor + 2].toInt() and 0xf8) == 0xf8) {
        "Invalid AVC extension reserved bits"
      }
      cursor += 3
      val extensionCount = data[cursor++].toInt() and 0xff
      repeat(extensionCount) { readParameterSet(13) }
    }
    require(cursor == data.size) { "Trailing AVC configuration bytes" }
    return output.toByteArray()
  }

  private val HIGH_PROFILES = setOf(100, 110, 122, 144, 44, 83, 86, 118, 128, 138, 139, 134, 135)
}
