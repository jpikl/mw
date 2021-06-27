# 🗃 Examples

> ℹ️ Use `rew --explain <pattern>` to print detailed explanation what a certain pattern does.

## Path processing

Print contents of the current working directory as absolute paths.

```bash
rew '{a}' *
```

The previous `*` shell expansion would not work for an empty directory.
As a workaround, we can read paths from standard input.

```bash
dir | rew '{a}'
```

## Batch rename

Rename all `*.jpeg` files to `*.jpg`.

```bash
find -name '*.jpeg' | rew -d '{B}.jpg' | mvb -v
```

The same thing but we generate and execute shell code.

```bash
find -name '*.jpeg' | rew -q 'mv -v {} {B}.jpg' | sh
```

Normalize base names of files to `file_001`, `file_002`, ...

```bash
find -type f | rew -d '{d}/file_{C|<3:0}{E}' | mvb -v
```

Flatten directory structure `./dir/subdir/` to `./dir_subdir/`.

```bash
find -mindepth 2 -maxdepth 2 -type d | rew -d '{D}_{F}' | mvb -v
```

## Batch copy

Make backup copy of each `*.txt` file with `.txt.bak` extension in the same directory.

```bash
find -name '*.txt'  | rew -d '{}.bak'  | cpb -v
```

Copy `*.txt` files to the `~/Backup` directory. Preserve directory structure.

```bash
find -name '*.txt'  | rew -d "$HOME/Backup/{p}"  | cpb -v
```

The same thing but with collapsed output directory structure.

```bash
find -name '*.txt'  | rew -d "$HOME/Backup/{f}"  | cpb -v
```

The same thing but we also append randomly generated base name suffix to avoid collisions.

```bash
find -name '*.txt'  | rew -d "$HOME/Backup/{b}_{U}.{e}"  | cpb -v
```

## Text processing

Normalize line endings in a file to `LF`

````bash
rew <input.txt >output.txt # LF is the default output terminator
````

Normalize line endings in a file to `CR+LF`.

````bash
rew -T$'\r\n' <input.txt >output.txt
````

Replace tabs with 4 spaces.

````bash
rew '{R:%t:    }' <input.txt >output.txt
````

That would also normalize line endings.
To prevent such behaviour, we can process the text as a whole.

````bash
rew -rR '{R:%t:    }' <input.txt >output.txt
````

Print the first word from each line in lowercase and with removed diacritics (accents).

```bash
rew '{=\S+|v|i}' <input.txt
```

## CSV editing

Swap the first and second column in a CSV file.

```bash
rew -e'([^,]*),([^,]*),(.*)' '{$2},{$1},{$3}' <input.csv >output.csv
```

The same thing but we use regex replace filter.

```bash
rew '{s/([^,]*),([^,]*),(.*)/$2,$1,$3}' <input.csv >output.csv
```
