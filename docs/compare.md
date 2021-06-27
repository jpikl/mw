# 🔬 Comparison

Let us compare `rew` to a variety of existing tools.

## rename

Both `rename` abd `rew` can be used to rename multiple files.

`rename` requires all inputs to be passed as arguments. 
This means you have to use `xargs` when processing output of `find`.
`rew` can read values directly from standard input.

Additionally, `rew` is only a text-processing tool and cannot rename files by itself.
You have to use accompanying `mvb` / `cpb` utilities, or you can generate and execute shell code.

```bash
find -name '*.jpeg' | xargs rename .jpeg .jpg     # Rename *.jpeg files to *.jpg
find -name '*.jpeg' | rew -d '{B}.jpg' | mvb      # The same thing using rew + mvb
find -name '*.jpeg' | rew -q 'mv {} {B}.jpg' | sh # The same thing using rew + mv + sh
```

## dirname

Both `dirname` and `rew` can remove last component from a path:

```bash
dirname 'dir/file.txt'   # Will print "dir"
rew '{d}' 'dir/file.txt' # The same thing using rew
```

## basename

Both `basename` and `rew` can remove leading directories from a path:

```bash
basename 'dir/file.txt'  # Will print "file.txt"
rew '{f}' 'dir/file.txt' # The same thing using rew
```

`basename` can additionally remove filename extension, but we have to manually provide it as a suffix.
`rew` is able to remove filename extension automatically:

```bash
basename 'dir/file.txt' '.txt' # Will print "file"
rew '{b}' 'dir/file.txt'       # The same thing using rew
```

In case the suffix does not represent an extension, `rew` requires an additional filter to remove it:

```bash
basename 'dir/file_txt' '_txt'   # Will print "file"
rew '{f|s:_txt$}' 'dir/file_txt' # The same thing using rew
```

## realpath

Both `realpath` and `rew` can resolve canonical form of a path:

```bash
realpath -e '/usr/../home' # Will print "/home"
rew '{P}' '/usr/../home'   # The same thing using rew
```

Or they can both compute a relative path:

```bash
realpath --relative-to='/home' '/usr' # Will print "../usr"
rew -w '/home' '{A}' '/usr'           # The same thing using rew
```

## pwd

Both `pwd` and `rew` can print the current working directory:

```bash
pwd                     # pwd is obviously easier to use
rew '{w}' '<any value>' # rew requires an additional input
```

## sed

Both `sed` and `rew` can replace text matching a regular expression:

```bash
echo '12 ab 34' | sed -E 's/([0-9]+)/_\1_/g' # Will print "_12_ ab _34_"
echo '12 ab 34' | rew '{S:(\d+):_$1_}'       # The same thing using rew
```

## cut

Both `cut` and `rew` can print substring:

```bash
echo 'abcde' | cut -c '2-4' # Will print "bcd"
echo 'abcde' | rew '{#2-4}' # The same thing using rew
```

Or they can both print fields:

```bash
echo 'ab,cd,ef' | cut -d',' -f2    # Will print "cd"
echo 'ab,cd,ef' | rew -s',' '{&2}' # The same thing using rew
```

## awk

`awk` is obviously a more powerful tool than `rew`.
However, there are some use cases where `rew` can replace `awk` using more compact pattern syntax.

Printing substring:

```bash
echo 'abcde' | awk '{print substr($0,2,3)}' # Will print "bcd"
echo 'abcde' | rew '{#2+3}'                 # The same thing using rew
```

Printing field:

```bash
echo 'ab,cd,ef' | awk -F',' '{print $2}' # Will print "cd"
echo 'ab,cd,ef' | rew -s',' '{&2}'       # The same thing using rew
```

Printing first match of a regular expression:

```bash
echo 'ab 12 cd' | awk 'match($0,/[0-9]+/) {print substr($0,RSTART,RLENGTH)}' # Will print "12"
echo 'ab 12 cd' | rew '{=\d+}'                                               # The same thing using rew
```

## grep

Both `grep` and `rew` can print matches of a regular expression:

```bash
echo 'ab 12 cd' | grep -Po '\d+' # Will print "12"
echo 'ab 12 cd' | rew '{=\d+}'   # The same thing using rew
```

If an input line contains multiple matches, `grep` will print each on a separate line.
`rew` will, however, print only the first match from each line.
This is because `rew` transforms lines in 1-to-1 correspondence.

In this particular case, we can workaround it, using raw output mode `-R` and regex replace filters `sS`.

```bash
echo '12 ab 34' | grep -Po '\d+'                        # Will print "12" and "34"
echo '12 ab 34' | rew -R '{s:^\D+$|S:\D*(\d+)\D*:$1%n}' # The same thing using rew
```
