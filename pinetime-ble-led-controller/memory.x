MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    /* These values correspond to the NRF52832 with SoftDevices S132 7.3.0
     * See blobs/nrf-softdevice/s132_nrf52_7.3.0/s132_nrf52_7.3.0_release-notes.pdf
     * and https://github.com/embassy-rs/nrf-softdevice */
    FLASH : ORIGIN = 0x00000000 + 152K, LENGTH = 512K - 152K
    RAM : ORIGIN = 0x20000000 + 18K, LENGTH = 64K - 18K
}