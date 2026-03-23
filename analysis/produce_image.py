from argparse           import ArgumentParser
from concurrent.futures import ProcessPoolExecutor
from multiprocessing    import Process
from pandas.errors      import PerformanceWarning
from pathlib            import Path
from tables             import open_file
from warnings           import catch_warnings
from warnings           import simplefilter

from utils import imgs_from_file


def signal(s):
    s = s.lower()
    if s not in "s1 s2".split():
        raise ValueError("Signal must be 'S1' or 'S2'")
    return s

def process(infile, outfile, signal, far_plane):
    df = imgs_from_file(infile, signal, far_plane)
    with catch_warnings():
        simplefilter("ignore", FutureWarning)
        simplefilter("ignore", PerformanceWarning)
        df.to_hdf(outfile, "/imgs", mode="w", complib="zlib", complevel=4)

    with ( open_file( infile, "r") as src
         , open_file(outfile, "a") as dst):
        dst.create_group(dst.root, "MC", createparents=True)
        src.copy_node(src.root.MC.config, dst.root.MC, "config")

def main():
    parser = ArgumentParser()
    parser.add_argument("-i", "--inputs"  , type=Path)
    parser.add_argument("-o", "--outputs" , type=Path)
    parser.add_argument("-s", "--signal"  , type=signal)
    parser.add_argument("--far-plane"     , action="store_true")
    parser.add_argument("--overwrite"     , action="store_true")
    parser.add_argument("--folder"        , action="store_true")
    parser.add_argument("-j", "--nprocess", type=int, default=12)

    args = parser.parse_args()

    # interpret inputs/outputs as a folder
    if args.folder:
        with ProcessPoolExecutor(max_workers=args.nprocess) as ex:
            for ifile in sorted(args.inputs.glob("*.h5")):
                ofile = args.outputs / ifile.name
                if ofile.exists() and not args.overwrite:
                    raise RuntimeError("Output file already exists. Use --overwrite to overwrite it")    
                ex.submit(process, ifile, ofile, args.signal, args.far_plane)
            
    else:
        if args.output_file.exists() and not args.overwrite:
            raise RuntimeError("Output file already exists. Use --overwrite to overwrite it")    
        process(args.input_file, args.output_file, args.signal, args.far_plane)
        
if __name__ == "__main__":
    main()