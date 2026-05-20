#!/usr/bin/env python3

from sys         import argv
from pathlib     import Path
from argparse    import ArgumentParser
from dataclasses import dataclass, field

import numpy  as np
import tables as tb

@dataclass
class Event:
    number: int = -1
    time: int = -1
    waveforms : list = field(default_factory=list)


def read(filename, channels=None):
    events = []

    event = None
    for i, line in enumerate(open(filename)):
        print(f"\rEvent {i}", end="", flush=True)
        tokens = (line.strip()
                      .replace("\t", " ")
                      .split(" ")
                 )

        if tokens[0] == "Event":
            if i:
                event.waveforms = np.array(event.waveforms).T
                events.append(event)

            event = Event()
            event.number = int(tokens[-1])
            continue

        if tokens[0] == "TimeStamp:":
            event.time = int(tokens[-1])
            continue

        if tokens[0] in  "S Samples:".split():
            continue

        if tokens[-1] == "us":
            sampling_time = float(tokens[-2])
            continue

        tokens = [tokens[c] for c in channels] if channels else tokens[1:]
        event.waveforms.append(list(map(float, tokens)))

    wflen = len(events[0].waveforms[0])
    time  = np.arange(wflen) * sampling_time

    waveforms = np.array([e.waveforms for e in events])
    events    = np.array([[e.number, e.time] for e in events])
    return events, waveforms, time


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("-i", "--input_folder" , type=Path , help="Input path" , required=True)
    parser.add_argument("-o", "--output_file"  , type=Path , help="Output file", default=None)
    parser.add_argument("-c", "--channels"     , type=int  , help="channels to store", nargs="*", required=True)
    parser.add_argument("--overwrite", action="store_true")

    args = parser.parse_args(argv[1:])
    compression = tb.Filters(complib="zlib", complevel=4)

    if args.output_file.exists() and not args.overwrite:
        raise RuntimeError("Output file exists. Use --overwrite to overwrite it")

    filenames = sorted(args.input_folder.glob("*.txt"))
    n         = len(filenames)

    channels = [c+1 for c in args.channels]

    with tb.open_file(args.output_file, mode="w", filters=compression) as file:
        events, waveforms, time = read(filenames.pop(0), channels)

        evt_store = None
        wfm_store = None
        for i, filename in enumerate(filenames):
            print(f"\nFile {filename}")

            events, waveforms, time = read(filename, channels)
            if i==0:
                file.create_array (file.root, "time", time.astype(np.float32), "Waveform time")

                evt_store = file.create_earray( file.root, "events"
                                              , atom=tb.UInt32Atom()
                                              , shape = (0,2)
                                              , title = "Event numbers and timestamps"
                                              )

                wfm_store = file.create_earray( file.root, "waveforms"
                                              , atom=tb.Float32Atom()
                                              , shape = (0, len(channels), len(time))
                                              , title = "Waveform amplitudes"
                                              )

            evt_store.append(events)
            wfm_store.append(waveforms)
