-- Three segment.next chords (Cmd+Opt+Y), gap seconds apart; prints the epoch seconds just before each keystroke.
on run argv
	set gap to (item 1 of argv) as real
	set stamp to "perl -MTime::HiRes=time -e 'printf q(%.4f), time'"
	set out to ""
	tell application "System Events"
		repeat with i from 1 to 3
			set t to do shell script stamp
			keystroke "y" using {command down, option down}
			set out to out & t & " "
			delay gap
		end repeat
	end tell
	return out
end run
