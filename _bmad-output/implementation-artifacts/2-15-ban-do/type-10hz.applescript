-- 300 ASCII keys at a fixed delay from one process, then Cmd+Return with the caret still in the cell.
-- Prints the epoch seconds just before key 1 and just before key 300 (the other keys are interpolated).
on run argv
	set gap to (item 1 of argv) as real
	set chars to "asdfghjkl "
	set stamp to "perl -MTime::HiRes=time -e 'printf q(%.4f), time'"
	set t1 to ""
	set t300 to ""
	tell application "System Events"
		repeat with i from 1 to 300
			if i = 1 then set t1 to do shell script stamp
			if i = 300 then set t300 to do shell script stamp
			keystroke (character (((i - 1) mod 10) + 1) of chars)
			if i < 300 then delay gap
		end repeat
		key code 36 using command down
	end tell
	return t1 & " " & t300
end run
