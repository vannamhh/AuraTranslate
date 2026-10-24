-- One process, fixed gap between keys (10 keys/s at gap 0.1). Each unit ends in "[n]"; the stamp for n is taken just before "]" is typed.
-- Log line: "n epoch unit-length". Runs until DUR seconds have elapsed.
on run argv
	set gap to (item 1 of argv) as real
	set dur to (item 2 of argv) as real
	set logf to item 3 of argv
	set chunks to {"troi hom nay ", "ong lao nhin ra bien ", "gio thoi qua rang tre ", "nang cuoi ", "han buoc di cham rai ", "khong ai tra loi "}
	set stamp to "perl -MTime::HiRes=time -e 'printf q(%.4f), time'"
	set t0 to (do shell script stamp) as real
	set n to 0
	tell application "System Events"
		repeat
			if ((do shell script stamp) as real) - t0 >= dur then exit repeat
			set n to n + 1
			set w to item (((n mod (count of chunks)) + 1)) of chunks
			set body to w & "[" & n
			repeat with i from 1 to (count of characters of body)
				keystroke (character i of body)
				delay gap
			end repeat
			set ts to do shell script stamp
			do shell script "echo " & n & " " & ts & " " & ((count of characters of body) + 1) & " >> " & quoted form of logf
			keystroke "]"
			delay gap
		end repeat
	end tell
	return n
end run
