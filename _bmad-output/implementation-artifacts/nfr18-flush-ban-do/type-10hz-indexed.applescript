use framework "Foundation"
use scripting additions

-- One process, fixed gap between keys (10 keys/s at gap 0.1). Each unit ends in "[n]"; the stamp for n is taken just before "]" is typed.
-- Log line: "n epoch unit-length". Runs until DUR seconds have elapsed.
-- Every key is also stamped in memory (NSDate, no process spawn) and written to <log>.keys when typing ends: a "#t0 <epoch>" header, then "seconds-since-t0 codepoint".
on run argv
	set gap to (item 1 of argv) as real
	set dur to (item 2 of argv) as real
	set logf to item 3 of argv
	set chunks to {"troi hom nay ", "ong lao nhin ra bien ", "gio thoi qua rang tre ", "nang cuoi ", "han buoc di cham rai ", "khong ai tra loi "}
	set stamp to "perl -MTime::HiRes=time -e 'printf q(%.4f), time'"
	set t0str to do shell script stamp
	set t0 to t0str as real
	set e0 to my nowEpoch()
	set n to 0
	set keyRows to {}
	tell application "System Events"
		repeat
			if ((do shell script stamp) as real) - t0 >= dur then exit repeat
			set n to n + 1
			set w to item (((n mod (count of chunks)) + 1)) of chunks
			set body to w & "[" & n
			repeat with i from 1 to (count of characters of body)
				keystroke (character i of body)
				set end of keyRows to (((my nowEpoch()) - e0) as text) & " " & (ASCII number (character i of body))
				delay gap
			end repeat
			set ts to do shell script stamp
			do shell script "echo " & n & " " & ts & " " & ((count of characters of body) + 1) & " >> " & quoted form of logf
			keystroke "]"
			set end of keyRows to (((my nowEpoch()) - e0) as text) & " " & (ASCII number "]")
			delay gap
		end repeat
	end tell
	set AppleScript's text item delimiters to linefeed
	set blob to keyRows as text
	set AppleScript's text item delimiters to ""
	do shell script "printf '%s\n%s' " & quoted form of ("#t0 " & t0str) & " " & quoted form of blob & " > " & quoted form of (logf & ".keys")
	return n
end run

on nowEpoch()
	return (current application's NSDate's |date|()'s timeIntervalSince1970()) as real
end nowEpoch
