#!/usr/bin/env bash
#
#   git-dit - the distributed issue tracker for git
#   Copyright (C) 2016 Matthias Beyer <mail@beyermatthias.de>
#   Copyright (C) 2016 Julian Ganz <neither@nut.email>
#
#   This program is free software; you can redistribute it and/or modify
#   it under the terms of the GNU General Public License version 2 as
#   published by the Free Software Foundation.
#
# -----
#
# $(basename $0)
#
# (c) 2016 Matthias Beyer, Julian Ganz

abort() {
    echo "$*" >&2
    exit 1
}

VERSION="0.1.0"

help() {
    cat <<EOS
    Usage: git dit [--version | -h | --help | <command>] <args...>

    git dit - the distributed issue tracker for git

    git-dit is free software. It is released under the terms of GPLv2
    (c) 2016 Julian Ganz, Matthias Beyer
EOS
}

main() {
    case $1 in
        --version)
            echo $VERSION
            exit 0
            ;;

        --help | -h)
            help
            exit 0
            ;;

        *)
			# Naive...
            local cmd=$1; shift
            exec git-dit-${cmd} $*
        ;;
    esac
}

main $*

