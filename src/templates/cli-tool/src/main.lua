#!/usr/bin/env lua
-- {{project_name}} - CLI tool
-- Version: {{project_version}}

local argparse = require("argparse")

local parser = argparse("{{project_name}}", "{{project_name}} - A CLI tool")
parser:argument("input", "Input file or value")
parser:option("-o --output", "Output file"):default("output.txt")
parser:flag("-v --verbose", "Verbose output")
parser:flag("-h --help", "Show help"):action(function()
    parser:help()
    os.exit(0)
end)

local args = parser:parse()

if args.verbose then
    print("{{project_name}} v{{project_version}}")
    print("Input:", args.input)
    print("Output:", args.output)
end

-- Your CLI tool logic here
print("Processing:", args.input)

