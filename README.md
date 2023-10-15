# bevy_mod_yarn

This is a bevy library for Yarn Spinner, based on the
[`yarnham`](https://github.com/mystal/yharnam) parser. Currently this requires
my fork of yharnam found here on the [`main
branch`](https://github.com/will-hart/yharnam). This branch adds support for
v2.3+ of yarn spinner and implements a bunch of missing functionality.

## Usage

See the examples directory for usage (minimal and "kitchen sink" examples are
available).

Note that **Yharnam only handles compiled yarn files**. Yarn files can be
compiled using the [Yarn Spinner
Console](https://github.com/YarnSpinnerTool/YarnSpinner-Console) which must be
downloaded separately.

The build script (`build.rs`) in this repository shows an example of how you can
automatically compile your yarn files during the build. Note that due to
limitations in bevy's asset loader the csv files created by the yarn compiler
need to be renamed from the Yarn compiler defaults.

## Features

The current feature(s) are available, enabled by default:

- `input-handlers`, which includes default input handlers (space to progress
  through dialogues and numbers 1-3 for selecting options). There are checks to
  ensure a valid option is pressed. To disable this and provider your own input
  handlers, set `default-features = false`.
