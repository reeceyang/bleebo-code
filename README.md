```
   ____     _     U _____ uU _____ u   ____     U  ___ u       ____   U  ___ u  ____  U _____ u 
U | __")u  |"|    \| ___"|/\| ___"|/U | __")u    \/"_ \/    U /"___|   \/"_ \/ |  _"\ \| ___"|/ 
 \|  _ \/U | | u   |  _|"   |  _|"   \|  _ \/    | | | |    \| | u     | | | |/| | | | |  _|"   
  | |_) | \| |/__  | |___   | |___    | |_) |.-,_| |_| |     | |/__.-,_| |_| |U| |_| |\| |___   
  |____/   |_____| |_____|  |_____|   |____/  \_)-\___/       \____|\_)-\___/  |____/ u|_____|  
 _|| \\_   //  \\  <<   >>  <<   >>  _|| \\_       \\        _// \\      \\     |||_   <<   >>  
(__) (__) (_")("_)(__) (__)(__) (__)(__) (__)     (__)      (__)(__)    (__)   (__)_) (__) (__) 
```

# Bleebo Code
*agent orchestrator built on jj*

a set of tui mini-apps and a server that automates zellij to display them

workspace browser:
- manage workspaces
- create workspaces from main (or any change)
- select a workspace to set it as the active workspace
- implementation:
  - read config from some file
  - when creating a new workspace, 
    - ask for slug and create a new folder in configured directory
    - zellij create new tab in directory, set the layout, and switch to tab
    - write to state file tab corresponding to workspace
  - indicate what workspace corresponds to the current working directory
  - to switch workspaces,
    - switch to tab corresponding to workspace
    - if the tab doesn't exist, recreate it
  - watch the state file and rerender if it changes
  - to delete workspace
    - update state file
    - delete directory (hopefully the process keeps running)
    - close tabs

file browser:
- browse files in a workspace
- should show directories/files with changes
- i'm sure there's already a tui tool for this, but the main useful feature for me would be to copy file paths quickly
- maybe this could be a plugin
- maybe this could be nvim...

terminal panes
- terminal in the active workspace

terminal pane selector
- like zellij's default tab bar, but only shows terminal panes in the current workspace 
- idk if this is even necessary tbh. maybe we just run zellij inside zellij lmao

diff viewer
- view the current changed files and review the diff against main (or any change)
- a little bit unsure about this one, bc you could just run jj diff in a terminal
- this could just be 2 zellij run panes with jj diff and jj st

jj log
- set the base branch
- manage prs
- shortcuts for editing changes, new change, squashing, describing
- maybe rebasing and updating stale workspace (could also go in the workspace browser)

~~server~~
- receives active workspace selection from workspace browser
  - on change, runs zellij cli commands to update panes in the active session.
  - maybe it could be even simpler, where 1 tab = 1 workspace, and we just switch tabs
  - do we even need a server? the workspace browser app can just run the zellij commands itself
  - creating a new workspace opens a new tab with a new layout
  - workspace browser persists the active selection state to a shared file on disk

design principles
- let zellij handle all the hard work. we should try to do as little zellij automation as possible to keep things simple
