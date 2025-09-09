#!/bin/python3

import argparse
from dataclasses import dataclass
from enum import Enum
from functools import total_ordering
import os
from pathlib import Path
import shlex
import subprocess


class Verbose:
    def on_run_command(self, environment: dict[str, str], command: list[str], working_directory: Path):
        environment_part = ' '.join(f'{key}={shlex.quote(value)}' for key, value in environment.items())
        if environment_part:
            environment_part = f'{environment_part} '

        print(f'{working_directory}> {environment_part}{shlex.join(command)}')

    def on_command_result(self, returncode: int):
        print(f'< Finished with exit code {returncode}')


class NonVerbose:
    def on_run_command(self, environment: dict[str, str], command: list[str], working_directory: Path):
        pass

    def on_command_result(self, returncode: int):
        pass


class CommandRunner:
    def __init__(self, verbose: bool, working_directory: Path):
        self.verbose = Verbose() if args.verbose else NonVerbose()
        self.working_directory = working_directory
    
    def run(self, environment: dict[str, str], command: list[str]) -> int:
        self.verbose.on_run_command(environment, command, self.working_directory)

        full_environment = {**os.environ, **environment}

        returncode = subprocess.run(command, env=full_environment, cwd=self.working_directory).returncode
        self.verbose.on_command_result(returncode)
        return returncode


# TODO: Make abc
class Flasher:
    def flash(self, project: 'Project', runner: CommandRunner) -> int:
        raise NotImplementedError()


class Esp32C3Flasher(Flasher):
    def flash(self, project: 'Project', runner: CommandRunner) -> int:
        partition_table_path = Path(project.name) / "partitions.csv"
        binary_path = Path("target") / project.target / "release" / project.name

        return runner.run(
            environment={},
            command=[
                'espflash', 'flash',
                '--monitor',
                f'--partition-table={partition_table_path}',
                str(binary_path),
            ]
        )


class PineTimeFlasher(Flasher):
    def flash(self, project: 'Project', runner: CommandRunner) -> int:
        binary_path = Path("target") / project.target / "release" / project.name

        return runner.run(
            environment={},
            command=[
                'probe-rs', 'run',
                '--chip', 'nRF52832_xxAA',
                str(binary_path),
            ]
        )


class Rp2350Flasher(Flasher):
    def flash(self, project: 'Project', runner: CommandRunner) -> int:
        binary_path = Path("target") / project.target / "release" / project.name

        return runner.run(
            environment={},
            command=[
                'probe-rs', 'run',
                '--chip', 'RP235x',
                str(binary_path),
            ]
        )


@dataclass
class Project:
    name: str
    target: str
    channel: str
    check_include_tests: bool
    rustflags: list[str]
    environment: dict[str, str]
    build_args: list[str]
    flasher: Flasher

    def validate_config(self):
        failed = False

        for key,value in self.environment:
            if not value:
                print(f'Error: Environment variable')

    def __create_environment(self) -> dict[str, str]:
        environment = self.environment.copy()
        
        if self.rustflags:
            environment['RUSTFLAGS'] = ' '.join(self.rustflags)
        
        return environment

    def format(self, runner: CommandRunner) -> int:
        return runner.run(
            environment={},
            command=[
                'cargo', 'fmt'
            ],
        )

    def check(self, runner: CommandRunner) -> int:
        return runner.run(
            environment={},
            command=[
                'cargo',
                f'+{self.channel}',
                'clippy',
                *(['--tests'] if self.check_include_tests else []),
                '--release',
                f'--target={self.target}',
                *self.build_args,
                '--',
                '-Dwarnings'
            ],
        )

    def build(self, runner: CommandRunner) -> int:
        return runner.run(
            environment=self.__create_environment(),
            command=[
                'cargo',
                f'+{self.channel}',
                'build',
                '--release',
                f'--target={self.target}',
                *self.build_args,
            ]
        )

    def flash(self, runner: CommandRunner) -> int:
        return self.flasher.flash(self, runner)


RUSTFLAGS_ESP32C3 = [
    '-C', 'link-arg=-Tlinkall.x',
    '-C', 'force-frame-pointers',
]


ENVIRONMENT_ESP32C3 = {
    'ESP_LOG': 'info',
    'EMBASSY_EXECUTOR_TASK_ARENA_SIZE': '65536',
}


PROJECTS = {p.name: p for p in [
        Project(
            name='esp32c3-hello-world',
            target='riscv32imc-unknown-none-elf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=RUSTFLAGS_ESP32C3,
            environment=ENVIRONMENT_ESP32C3,
            build_args=[],
            flasher=Esp32C3Flasher(),
        ),
        Project(
            name='esp32c3-wifi-ble-controlled-led',
            target='riscv32imc-unknown-none-elf',
            channel='nightly',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=RUSTFLAGS_ESP32C3,
            environment=ENVIRONMENT_ESP32C3,
            build_args=['-Z', 'build-std=core,alloc'],
            flasher=Esp32C3Flasher(),
        ),
        Project(
            name='pinetime-ble-led-controller',
            target='thumbv7em-none-eabihf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[
                '-C', 'link-arg=-Tlink.x',
                '-C', 'link-arg=-Tdefmt.x',
                # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
                # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
                # '-C', 'link-arg=--nmagic',
            ],
            environment={
                'DEFMT_LOG': 'trace',
            },
            build_args=[],
            flasher=PineTimeFlasher(),
        ),
        Project(
            name='rp2350-hello-world',
            target='thumbv8m.main-none-eabihf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[],
            environment={
                'DEFMT_LOG': 'debug',
            },
            build_args=[],
            flasher=Rp2350Flasher(),
        ),
        Project(
            name='rp2350-apds9960-distance-light',
            target='thumbv8m.main-none-eabihf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[],
            environment={
                'DEFMT_LOG': 'debug',
            },
            build_args=[],
            flasher=Rp2350Flasher(),
        ),
        Project(
            name='rp2350-apds9960-nightlight',
            target='thumbv8m.main-none-eabihf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[],
            environment={
                'DEFMT_LOG': 'debug',
            },
            build_args=[],
            flasher=Rp2350Flasher(),
        ),
        Project(
            name='rp2350-apds9960-toggle-lamp',
            target='thumbv8m.main-none-eabihf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[],
            environment={
                'DEFMT_LOG': 'debug',
            },
            build_args=[],
            flasher=Rp2350Flasher(),
        ),
        Project(
            name='rp2350-apds9960-toggle-lamp-interrupt',
            target='thumbv8m.main-none-eabihf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[],
            environment={
                'DEFMT_LOG': 'debug',
            },
            build_args=[],
            flasher=Rp2350Flasher(),
        ),
        Project(
            name='rp2350-wifi-ble-controlled-led',
            target='thumbv8m.main-none-eabihf',
            channel='nightly',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
            rustflags=[],
            environment={
                'DEFMT_LOG': 'debug',
            },
            build_args=['-Z', 'build-std=core'],
            flasher=Rp2350Flasher(),
        )
    ]
}


@total_ordering
class Stage(Enum):
    FORMAT = 0
    CHECK = 1
    BUILD = 2
    FLASH = 3

    def __lt__(self, other):
        if self.__class__ is other.__class__:
            return self.value < other.value
        return NotImplemented


ALLOWED_STAGES = [stage.name.lower() for stage in Stage]


def parse_stage(stage_str: str) -> Stage:
    try:
        return Stage[stage_str.upper()]
    except KeyError:
        allowed_stages = ', '.join(ALLOWED_STAGES)
        raise ValueError(f'Invalid stage "{stage_str}". Allowed stages are: {allowed_stages}')


def ensure_success(returncode: int):
    if returncode != 0:
        exit(returncode)


if __name__ == '__main__':
    try:
        parser = argparse.ArgumentParser(description='Run build stages for a given project.')
        parser.add_argument(
            '--up-to-stage',
            choices=ALLOWED_STAGES,
            default=ALLOWED_STAGES[-1],
            help="Optional stage to run up to (inclusive)."
        )
        parser.add_argument(
            '-v',
            '--verbose',
            action='store_true',
            help='Verbose mode, output executed commands commands'
        )
        parser.add_argument(
            'project',
            type=str,
            help='Name of the project'
        )

        args = parser.parse_args()

        project = PROJECTS.get(args.project)
        if not project:
            available_projects = ', '.join(PROJECTS.keys())
            print(f'Error: No such project "{args.project}". Valid options are: {available_projects}')
            exit(1)

        try:
            stage = parse_stage(args.up_to_stage)
        except ValueError as e:
            print(str(e))
            exit(1)

        working_directory = Path(__file__).resolve().parent / project.name
        runner = CommandRunner(args.verbose, working_directory)

        print(f'### Building project {project.name} ###')

        if stage >= Stage.FORMAT:
            print(f'### Format ###')
            ensure_success(project.format(runner))
        if stage >= Stage.CHECK:
            print(f'### Check ###')
            ensure_success(project.check(runner))
        if stage >= Stage.BUILD:
            print(f'### Build ###')
            ensure_success(project.build(runner))
        if stage >= Stage.FLASH:
            print(f'### Flash ###')
            ensure_success(project.flash(runner))
    except KeyboardInterrupt:
        print('### Aborted ###')
        exit(1)
