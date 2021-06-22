# 💡 Overview

`rew` is a text processing CLI tool that rewrites FS paths according to a pattern.

## How rew works

1. Reads values from standard [input](input).
2. Rewrites them according to a [pattern](pattern).
3. Prints results to standard [output](output).

![How rew works](images/diagram.svg)

Input values are assumed to be FS paths, however, `rew` is able to process any UTF-8 encoded text.

```bash
find -iname '*.jpeg' | rew 'img_{C}.{e|l|r:e}'
```

`rew` is also distributed with two accompanying utilities (`mvb` and `cpb`) which move/copy files and directories, based on `rew` output.

```bash
find -iname '*.jpeg' | rew 'img_{C}.{e|l|r:e}' -d | mvb
```

