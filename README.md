# Dotman

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

```yaml
taskgroups:
  normalfile_common:
  - type: cp
    src: "alacritty/"
    dest: "$XDG_CONFIG_HOME/alacritty/"
    templates:
      - target: alacritty/alacritty.yml
        vars:
          font_size: 11

scenarios:
- name: "sakanainu"
  match:
  - hostname: "^sakanainu$"
  tasks:
  - normalfile_common
- name: "my desktop"
  match:
  - hostname: "^kanimogura$"
  tasks:
  - normalfile_common
```


# License

[The Unlicense](https://unlicense.org/)
