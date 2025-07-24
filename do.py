#!/bin/python3

import argparse
from dataclasses import dataclass
from enum import Enum
from functools import total_ordering
from pathlib import Path
import shlex
import subprocess


class Verbose:
    def on_run_command(self, command: list[str], working_directory: Path):
        print(f'{working_directory}> {shlex.join(command)}')

    def on_command_result(self, returncode: int):
        print(f'< Finished with exit code {returncode}')


class NonVerbose:
    def on_run_command(self, command: list[str], working_directory: Path):
        pass

    def on_command_result(self, returncode: int):
        pass


class CommandRunner:
    def __init__(self, verbose: bool, working_directory: Path):
        self.verbose = Verbose() if args.verbose else NonVerbose()
        self.working_directory = working_directory
    
    def run(self, command: list[str]) -> int:
        self.verbose.on_run_command(command, self.working_directory)
        returncode = subprocess.run(command, cwd=self.working_directory).returncode
        self.verbose.on_command_result(returncode)
        return returncode


@dataclass
class Project:
    name: str
    target: str
    channel: str
    check_include_tests: bool

    def format(self, runner: CommandRunner) -> int:
        return runner.run([
            'cargo', 'fmt',
            '--package', self.name,
        ])

    def check(self, runner: CommandRunner) -> int:
        return runner.run([
            'cargo',
            f'+{self.channel}',
            'clippy',
            *(['--tests'] if self.check_include_tests else []),
            '--release',
            f'--target={self.target}',
            '--package', self.name,
            '--',
            '-Dwarnings'
        ])

    def build(self, runner: CommandRunner) -> int:
        return runner.run([
            'cargo',
            f'+{self.channel}',
            'build',
            '--release',
            f'--target={self.target}',
            '--package', self.name,
        ])

    def flash(self, runner: CommandRunner) -> int:
        partition_table_path = Path(self.name) / "partitions.csv"
        binary_path = Path("target") / self.target / "release" / self.name

        return runner.run([
            'espflash', 'flash',
            '--monitor',
            f'--partition-table={partition_table_path}',
            str(binary_path)
        ])


PROJECTS = {p.name: p for p in [
        Project(
            name='esp32c3-hello-world',
            target='riscv32imc-unknown-none-elf',
            channel='stable',
            # This should usually be enabled but clippy complains as there are no tests.
            check_include_tests=False,
        ),
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

    working_directory = Path(__file__).resolve().parent
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
