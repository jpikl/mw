# 🆎 Substring filters

| Filter  | Description                                       |
| ------- | ------------------------------------------------- |
| `#A-B`  | Substring from index `A` to `B`.<br><small>Indices `A`, `B` start from 1 and are both inclusive.<br>Use `-A` for backward indexing.</small> |
| `#A+L`  | Substring from index `A` of length `L`.           |
| `#A-`   | Substring from index `A` to end.                  |
| `#A`    | Character at index `A`.<br><small>Equivalent to `#A-A`.</small>  |

Examples:

| Input   | Pattern  | Output | | Input   | Pattern   | Output |
| ------- | -------- | ------ |-| ------- | --------- | ------ |
| `abcde` | `{#2-3}` | `bc`   | | `abcde` | `{#-2-3}` | `cd`   |
| `abcde` | `{#2+3}` | `bcd`  | | `abcde` | `{#-2+3}` | `bcd`  |
| `abcde` | `{#2-}`  | `bcde` | | `abcde` | `{#-2-}`  | `abcd` |
| `abcde` | `{#2}`   | `b`    | | `abcde` | `{#-2}`   | `d`    |
