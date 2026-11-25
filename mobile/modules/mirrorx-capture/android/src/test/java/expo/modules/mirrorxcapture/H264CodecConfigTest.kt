package expo.modules.mirrorxcapture

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class H264CodecConfigTest {
  private val startCode = byteArrayOf(0, 0, 0, 1)

  @Test
  fun normalizesRawSpsAndPps() {
    val sps = byteArrayOf(0x67, 0x64, 0x00, 0x1f)
    val pps = byteArrayOf(0x68, 0xee, 0x3c)
    assertArrayEquals(startCode + sps + startCode + pps, H264CodecConfig.toAnnexB(listOf(sps, pps)))
  }

  @Test
  fun preservesAnnexBCodecSpecificData() {
    val config = byteArrayOf(0, 0, 0, 1, 0x67, 0x64, 0, 0x1f, 0, 0, 1, 0x68, 0xee)
    assertArrayEquals(config, H264CodecConfig.toAnnexB(listOf(config)))
  }

  @Test
  fun extractsSpsAndPpsFromAvccRecord() {
    val avcc = byteArrayOf(
      1, 100, 0, 31, 0xff.toByte(), 0xe1.toByte(),
      0, 3, 0x67, 0x64, 0x1f,
      1, 0, 2, 0x68, 0xee.toByte()
    )
    val expected = startCode + byteArrayOf(0x67, 0x64, 0x1f) +
      startCode + byteArrayOf(0x68, 0xee.toByte())
    assertArrayEquals(expected, H264CodecConfig.toAnnexB(listOf(avcc)))
  }

  @Test
  fun rejectsAvccWithMissingSpsOrPps() {
    val noSps = byteArrayOf(1, 100, 0, 31, 0xff.toByte(), 0xe0.toByte())
    val noPps = byteArrayOf(1, 100, 0, 31, 0xff.toByte(), 0xe1.toByte(), 0, 1, 0x67)
    assertThrows(IllegalArgumentException::class.java) {
      H264CodecConfig.toAnnexB(listOf(noSps))
    }
    assertThrows(IllegalArgumentException::class.java) {
      H264CodecConfig.toAnnexB(listOf(noPps))
    }
  }

  @Test
  fun rejectsUnsupportedCodecSpecificData() {
    assertThrows(IllegalArgumentException::class.java) {
      H264CodecConfig.toAnnexB(listOf(byteArrayOf(0x65, 0x01)))
    }
  }
}
