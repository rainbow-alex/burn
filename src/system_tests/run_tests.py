#!/usr/bin/env python
import sys
import os
from os import path
import subprocess
from difflib import Differ

SHOW_OUTPUT = False
EXIT_STATUS = 0

def run_tests( file_or_directory ):
	
	if path.isdir( file_or_directory ):
		
		for f in os.listdir( file_or_directory ):
			run_tests( path.join( file_or_directory, f ) )
		
	elif path.isfile( file_or_directory ):
		
		if file_or_directory[-9:] != ".burntest":
			return
		
		print "%s ..." % file_or_directory,
		
		source, expected_output = open( file_or_directory ).read().split( "\n/* OUTPUTS\n", 2 )
		
		process = subprocess.Popen(
			"build/bin/burn -q -",
			stdin = subprocess.PIPE,
			stdout = subprocess.PIPE,
			stderr = subprocess.STDOUT,
			shell = True
		)
		process.stdin.write( source )
		process.stdin.close()
		
		actual_output = process.stdout.read()
		
		if actual_output == expected_output:
			print "\033[32;1mOK\033[0m"
		
		else:
			global EXIT_STATUS
			EXIT_STATUS = 1
			print "\033[31;1mFAIL\033[0m"
			
			for line in Differ().compare( expected_output.splitlines(1), actual_output.splitlines(1) ):
				if line[0] != "?":
					print "\t" + line,
		
		if SHOW_OUTPUT:
			print actual_output,

args = sys.argv[1:]

if args and args[0] == "--show-output":
	args.pop(0)
	SHOW_OUTPUT = True

for arg in args or os.listdir( "." ):
	if not path.exists( arg ):
		print "File or directory does not exist: %s" % arg
	else:
		run_tests( arg )

sys.exit( EXIT_STATUS )
