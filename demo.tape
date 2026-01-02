
Set FontSize 32
Set Width 1000
Set Height 1000
Set Padding 40
Set FontFamily "JetBrains Mono"

# Start the game
Hide
Type "cargo run"
Enter
Show
Sleep 15s

# Play a bit to show state
Type "p"
Sleep 500ms
Type "1"
Sleep 500ms
Right
Sleep 200ms
Right
Sleep 200ms
Down
Sleep 200ms
Type "p"
Sleep 200ms
Type "4"
Sleep 500ms

# Take the screenshot
Screenshot assets/screenshot.png

# Quit
Type "q"
Sleep 500ms
