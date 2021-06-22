# 📊 Field filters

| Filter  | Description                                                     |
| ------- | --------------------------------------------------------------- |
| `&N:S`  | Split value using separator `S`, output `N`-th field.<br><small>Field indices `N` start from 1.<br>Use `-N` for backward indexing.<br>Any other character than `:` can be also used as a delimiter.<br>Use of `/` as a delimiter has special meaning (see below).</small> |
| `&N/S`  | Split value using regular expression `S`, output `N`-th field. |
| `&N`    | Split value using default separator, output `N`-th field.       |

The default field separator is *horizontal tab*.

- Use `-s, --separator` option to change it to a string.
- Use `-S, --separator-regex` option to change it to a regular expression.

```bash
echo a1-b2 | rew '{&1} {&2}' -s'-'       # Will print "a1 b2"
echo a1-b2 | rew '{&1} {&2}' -S'[^a-z]+' # Will print "a b"
```

Examples:

| Input    | Pattern        | Output    | | Input    | Pattern         | Output    |
| -------- | -------------- | --------- |-| -------- | --------------- | --------- |
| `a1\tb2` | `{&1}`         | `a1`      | | `a1\tb2` | `{&-1}`         | `b2`      |
| `a1\tb2` | `{&2}`         | `b2`      | | `a1\tb2` | `{&-2}`         | `a1`      |
| `a1--b2` | `{&1:-}`       | `a1`      | | `a1--b2` | `{&-1:-}`       | `b2`      |
| `a1--b2` | `{&2:-}`       | *(empty)* | | `a1--b2` | `{&-2:-}`       | *(empty)* |
| `a1--b2` | `{&3:-}`       | `b2`      | | `a1--b2` | `{&-3:-}`       | `a1`      |
| `a1--b2` | `{&1/[^a-z]+}` | `a`       | | `a1--b2` | `{&-1/[^a-z]+}` | *(empty)* |
| `a1--b2` | `{&2/[^a-z]+}` | `b`       | | `a1--b2` | `{&-2/[^a-z]+}` | `b`       |
| `a1--b2` | `{&3/[^a-z]+}` | *(empty)* | | `a1--b2` | `{&-3/[^a-z]+}` | `a`       |
