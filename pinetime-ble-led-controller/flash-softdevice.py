#!/bin/python3

from pathlib import Path
import subprocess


def erase_all() -> int:
    return subprocess.run([
        'probe-rs', 'erase',
        '--chip', 'nrf52832_xxAA',
        '--allow-erase-all',
    ]).returncode


def flash_softdevice(softdevice_hex_file: Path) -> int:
    return subprocess.run([
        'probe-rs', 'download',
        '--verify',
        '--binary-format', 'hex',
        '--chip', 'nRF52832_xxAA',
        str(softdevice_hex_file),
    ]).returncode


def ensure_success(returncode: int):
    if returncode != 0:
        exit(returncode)


if __name__ == '__main__':
    working_directory = Path(__file__).resolve().parent
    softdevice_hex_file = working_directory / 'blobs' / 'nrf-softdevice' / 's132_nrf52_7.3.0' / 's132_nrf52_7.3.0_softdevice.hex'

    ensure_success(erase_all())
    ensure_success(flash_softdevice(softdevice_hex_file))
