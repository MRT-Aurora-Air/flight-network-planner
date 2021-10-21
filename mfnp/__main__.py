import argparse
import os

def main():
    print(os.getcwd())
    from __init__ import __version__
    from getconfig import get_config
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(help='task to run', dest="task")

    subparsers.add_parser('info', help="Shows program info")
    subparsers.add_parser('help', help="View this page")

    p_getconfig = subparsers.add_parser('getconfig', help="Create a new configuration file")
    p_getconfig.add_argument('-o', '--output', type=str, help="The name of the file, excluding the file extension", default="config")

    p_run = subparsers.add_parser('run', help="Run the planner")
    p_run.add_argument('file', type=str, help="The name of the configuration file to read from")
    p_run.add_argument('-o', '--output', type=str, help="The file to output, excluding the file extension", default="plan")
    p_run.add_argument('-f', '--format', type=str, help="The format to output results as", default="json", choices=['json', 'yaml', 'txt'])
    p_run.add_argument('-v', '--verbose', help="Enable logging to console", action='store_true')

    args = parser.parse_args()

    if args.task == "info":
        print(f"MRT Flight Network Planner v{__version__}")
        print("Made by 7d for Aurora Air")
    elif args.task == "getconfig":
        get_config(args.output)
    elif args.task == "run":
        pass
    else:
        parser.print_help()

main()