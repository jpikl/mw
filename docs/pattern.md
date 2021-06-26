# ✏️ Pattern

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

```bash
rew --explain 'file_{c|<3:0}.{e}'
```

## Syntax

By default, pattern characters are directly copied to output.

| Input   | Pattern | Output |
| ------- | ------- | ------ |
| *(any)* | `abc`   | `abc`  |

Characters `{` and `}` form an expression which is evaluated and replaced in output.
Empty expression `{}` evaluates directly to input value.

| Input   | Pattern      | Output          |
| ------- | ------------ | --------------- |
| `world` | `{}`         | `world`         |
| `world` | `Hello, {}!` | `Hello, world!` |

Expression may contain one or more [filters](filters), separated by `|`.
Filters are consecutively applied on input value.

| Input      | Pattern         | Output     | Description                           |
| ---------- | --------------- | ---------- | ------------------------------------- |
| `old.JPEG` | `new.{e}`       | `new.JPEG` | Extension                             |
| `old.JPEG` | `new.{e|l}`     | `new.jpeg` | Extension, Lowercase                  |
| `old.JPEG` | `new.{e|l|r:e}` | `new.jpg`  | Extension, Lowercase, Remove&nbsp;`e` |

Use `-q, --quote` flag to automatically wrap  output of every expression in quotes.

```bash
echo abc | rew {}     # Will print  abc
echo abc | rew {} -q  # Will print 'abc'
echo abc | rew {} -qq # Will print "abc"
```

## Escaping

Character `%` starts an escape sequence.

| Sequence | Description                |
| -------- |--------------------------- |
| `%/`     | System directory separator<br><small>`\` on Windows<br>`/` everywhere else</small> |
| `%n`     | Line feed                  |
| `%r`     | Carriage return            |
| `%t`     | Horizontal tab             |
| `%0`     | Null                       |
| `%{`     | Escaped `{`                |
| `%|`     | Escaped `|`                |
| `%}`     | Escaped `{`                |
| `%%`     | Escaped `%`                |

Use `--escape` option to set a different escape character.

```bash
rew '{R:%t: }'              # Replace tabs with spaces
rew '{R:\t: }' --escape='\' # The same thing, different escape character
```
