# TimeSkwire
TimeSkwire is a lean and mean PDF reporting engine for
[TimeWarrior](https://taskwarrior.org/docs/timewarrior/). It aims to
integrate well with TimeWarrior's config and let you control it from the same config
file or override the settings with environment variables.

# Installation
Unless you're willing to contribute, it is best that you install TimeSkwire
using Cargo:

```shell
$ cargo install timeskwire
```

After installing the binary, TimeSkwire needs to hook up with your TimeWarrior
installation's extension directory:

```shell
# Specify extension_dir as the extension directory if necessary;
# ~/.timewarrior/extensions is the default
$ timeskwire init [extension_dir]
```

# Choosing reports
Once I get some additional report kinds done, you'll be able to permanently
choose a non-default one by inserting `timeskwire.report.kind = "<kind>"` into
your TimeWarrior config. For overriding that value you'll be using the
`TIMESKWIRE_REPORT=<kind>` env.

# Contributing
As of today, only one report is available. However, if you feel like
contributing to this project, check out the `src/reports` folder and add your
own reports using the `Report` trait defined in `mod.rs`. **All suggestions for
improving TimeSkwire are the most welcome** - don't hesitate to file an issue if
you can't contribute.
