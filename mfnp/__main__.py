import argparse
import blessed
import json, yaml
import os
term = blessed.Terminal()

def main():
    from __init__ import __version__
    from getconfig import get_config
    from run import run
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(help='task to run', dest="task")

    subparsers.add_parser('info', help="Shows program info")

    p_getconfig = subparsers.add_parser('getconfig', help="Create a new configuration file")
    p_getconfig.add_argument('-o', '--output', type=str, help="The name of the file, excluding the file extension", default="config")
    p_getconfig.add_argument('-f', '--force', help="Whether to force an overwrite, if there is an existing file of the same name", action='store_true')

    p_run = subparsers.add_parser('run', help="Run the planner")
    p_run.add_argument('file', type=str, help="The name of the configuration file to read from")
    p_run.add_argument('-o', '--output', type=str, help="The file to output, excluding the file extension", default="plan")
    p_run.add_argument('-f', '--format', type=str, help="The format to output results as", default="json", choices=['json', 'yaml', 'txt'])
    p_run.add_argument('-v', '--verbose', help="Makes planner output more info", action='count', default=0)
    p_run.add_argument('-q', '--quiet', help="Makes planner output less info", action='count', default=0)
    p_run.add_argument('-nw', '--nowarn', help="Suppress warnings", action="store_true")

    args = parser.parse_args()

    if args.task == "info":
        print(term.yellow(f"MRT Flight Network Planner v{__version__}"))
        print(term.yellow(f"Made by 7d for Aurora Air"))
    elif args.task == "getconfig":
        get_config(args.output, force=args.force)
    elif args.task == "run":
        with open(args.file, "r") as f:
            config = yaml.safe_load(f)
            f.close()
        output = run(config, format=args.format, verbosity=args.verbose-args.quiet, nowarn=args.nowarn)
        with open(args.output+".json", "w") as f:
            json.dump(output, f, indent=2)
            f.close()
    else:
        parser.print_help()

main()