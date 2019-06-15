# mb_ext_script
Extension to the Mount and Blade module system that allows one to write scripts in an alternative syntax in separate files.

  While I love playing Mount and Blade: Warband, I've never been fond of its "scripting" system. Whatever their reasons (probably understandable ones), the system the game devs came up with has given us a giant monolothic clumsy mess. Much of the syntax invovled is simply there because we have been stuck expressing our code in python lists and tuples, the many parentheses, brackets, and commas not actually making us any more productive, but rather the opposite. This python module is an attempt to alleviate that pain somewhat. 

  Using the awesome parser generator library [Pest](https://github.com/pest-parser/pest) and excellent Python interop library [cpython](https://github.com/dgrunwald/rust-cpython) this module can parse and translate an external .mbs script in a more efficient but very familiar syntax. It removes the need for much of the annoying extra parentheses, commas, brackets, quotations etc. without actually changing things so that those already experienced with modding M&B need to relearn how to script.

# Example

Let's show the Warband script for calculating the total wages the player must pay based on the troops in their party.

## Original:
```python
  ("game_get_total_wage",
    [
      (assign, ":total_wage", 0),
      (party_get_num_companion_stacks, ":num_stacks", "p_main_party"),
      (try_for_range, ":i_stack", 0, ":num_stacks"),
        (party_stack_get_troop_id, ":stack_troop", "p_main_party", ":i_stack"),
        (party_stack_get_size, ":stack_size", "p_main_party", ":i_stack"),
        (call_script, "script_game_get_troop_wage", ":stack_troop", 0),
        (val_mul, reg0, ":stack_size"),
        (val_add, ":total_wage", reg0),
      (try_end),
      (assign, reg0, ":total_wage"),
      (set_trigger_result, reg0),
  ]),
```

Ick.

## The equivalent mbs script
```
/* game_get_total_wage.mbs */
/*
 * C-style block comments are allowed, single line // comments however are not
 *
 * Notice how functionally little changes in the language, it simply aims to remove the pointless cruft
 * from before.
 */
assign :total_wage 0;
party_get_num_companion_stacks :num_stacks p.main_party;

try_for_range :i_stack 0 :num_stacks;
    party_stack_get_troop_id :stack_troop p.main_party :i_stack;
    party_stack_get_size :stack_size p.main_party :i_stack;
    call_script script.game_get_troop_wage :stack_troop 0;
    val_mul reg.0 :stack_size;
    val_add :total_wage reg.0;
try_end;

assign reg.0 :total_wage;
set_trigger_result reg.0;
```

```python
# The module is just a standalone .pyd and can be imported easily
import mb_ext_script

# mb_ext_script needs notified of what python modules you are using so that it can look
# up the proper values after parsing the mbs script. Failure to specify a needed
# module won't cause the parser to fail, but when you try to build your module
# the script will not compile.
modules = [
  'header_common',
  'header_operations',
  'module_constants',
  'header_parties',
  'header_skills',
  'header_mission_templates',
  'header_items',
  'header_triggers',
  'header_terrain_types',
  'header_music',
  'header_map_icons',
  'header_presentations',
  'ID_animations'
]

scripts = [
  # ...
  # Here we specify the script source file, the name of the script the game will see, and the modules needed to build the script.
  mb_ext_script.parse("game_get_total_wage.mbs", "game_get_total_wage", modules)
  # ...
]
```

# Summary of Important Differences

## Statement Structure
Scripts are composed of "statements". In the original modsys using Python, we write these as tuples such as
```python
(assign, reg0, "$cheat_mode")
```

In mbs syntax we simply write them out followed by a semicolon.
```
assign reg.0 $cheat_mode;
```

We can write long statements on multiple lines if we wish.
```
/* This is all in one statement. In mbs syntax we only care about whitespace to the extent
 * that the operation and parameters are separated by at least one space each. Beyond that
 * the parser doesn't care and will ignore it.
 */
party_stack_get_troop_id :stack_troop
  p.main_party :i_stack;
```

## Variables
Originally, we'd express variables as strings with the proper prefix, e.g. "$cheat_mode" is a global variable, and it would be
local if it was prefixed with ":" instead. In mbs syntax the quotes are no longer used.

### Local Variables
Just prefix it with a colon. For example the following are equivalent.
```python
":my_var"
```

```
:my_var
```

### Global Variables
There are two signatures for global variables. The direct way that can always be used is simply applying '$'.
```python
"$cheat_mode"
```

```
$cheat_mode
```

Many global variables are often prefixed with a "g_" however. You can if you wish simply write `$g_my_global`, however the following also works.

```
g.cheat_mode
```

Only use the `g.` syntax if the global you're using (is)/(will be) prefixed with "g_".

## Registers

### Standard registers
The following are equivalent.
```python
reg0
reg7
reg8
```

```
reg.0
reg.7
reg.8
```

### Position registers
The following are equivalent.
```python
pos0
pos7
pos8
```

```
pos.0
pos.7
pos.8
```

### String registers
The following are equivalent.
```python
s0
s7
s8
```

```
str.0
str.7
str.8
```

## Referencing things with ids.

When modding in python we sometimes have to reference a script or party etc. with strings such as 
"script_game_start" or "p_main_party". In mbs syntax again we don't use quotes anymore, but rather a prefix.

```
script.game_start
```

```
p.main_party
```

The prefix in mbs always aligns with the prefix used in the module system already. 

# Installation

Either download a binary or build this module from source into a .pyd yourself. To build it from source you'll need at least the following.
- [Rust](https://www.rust-lang.org/tools/install)
- [Python 2.7 - If you're a M&B modder then you should have this already.](https://www.python.org/download/releases/2.7/)
- On Windows make sure the python lib path is added to your LIBPATH environment variable. This is not the same as having Python added to Path.

Note, the binary you use should be compiled against the *exact* version of Python you're using. A binary built against 2.7.11 won't work
if you're running 2.7.16. I will try to keep releases out with Python updates promptly.

Once you have your .pyd whether you built it yourself or downloaded it, just paste it into your module system folder, then import it
from whichever python script you'd like. 

```
import mb_ext_script
```

The module only has two functions.

```
mb_ext_script.version() Simply details the version of mb_ext_script and the version Python it's running against

mb_ext_script.parse(path, name, modules) Attempts to parse the file specified by "path" and convert it into a M&B script ready to be compiled.
```


This extension is in its infancy, please feel free to submit issues and ask questions.
