btc-e.com TollBOX console viewer Rust implementation

Primary aim of this project, is to try Rust language, with adding new feature.

You should be able to view trollbox conversations in your terminal.
In addition  you should be able to read more then one chat "channel" | "room"
simultaneously. Also btc-e "ticks" volume will be displayed as part of chat.

Usage:
    wstest [-v <limit> | --volume=<limit>] [-c <arguments>... | --channels=<arguments>...] [-h | --help]

    Options:
        -h --help                  Show this screen.
        -v --volume=<limit>        Show tick price with volume > limit [default: 10.0].
                                   0 == Disable.
        -c --channels=<arguments>  Listen channels [default: chat_en chat_ru chat_ch].
