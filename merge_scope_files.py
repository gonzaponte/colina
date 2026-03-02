#!/usr/bin/env python3

from sys      import argv
from pathlib  import Path
from argparse import ArgumentParser
from operator import itemgetter

import numpy  as np
import tables as tb


def read(filename):
    return np.loadtxt(filename, delimiter=",", skiprows=5).T


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("-i", "--input_folder", type=Path, help="Input path" , required=True)
    parser.add_argument("-o", "--output_file" , type=Path, help="Output file", default=None)
    parser.add_argument("--overwrite", action="store_true")

    args = parser.parse_args(argv[1:])
    compression = tb.Filters(complib="zlib", complevel=4)

    if args.output_file.exists() and not args.overwrite:
        raise RuntimeError("Output file exists. Use --overwrite to overwrite it")

    first = True
    for channel in range(1, 5):
        channel   = f"C{channel}"
        filenames = sorted(args.input_folder.glob(f"{channel}*.csv"))
        n         = len(filenames)
        print(f"Channel {channel}: {n} files")
        if not n: continue

        wfs  = map(read, filenames)
        wfs  = map(itemgetter(1), wfs)
        wfs  = np.asarray(list(wfs))

        mode = "w" if first else "a"
        with tb.open_file(args.output_file, mode, filters=compression) as file:
            if first:
                time = read(filenames[0])[0]
                file.create_array(file.root, "time", time, "Waveform time")

            file.create_carray(file.root, channel,  obj=wfs, title="Waveform amplitudes")

        first = False
