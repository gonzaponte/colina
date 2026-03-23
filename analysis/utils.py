import numpy  as np
import pandas as pd
import matplotlib.pyplot as plt

class AttrDict(dict):
    def __getattr__(self, item):
        return self.__getitem__(item)

def sipm_positions(size, gap, nperside):
    """
    SiPM positions assuming a square `nperside`x`nperside`
    matrix with sipms of side-length `size` and and spaced
    by an amount `gap`
    """
    n      = nperside
    offset = n * (size + gap) / 2.
    xy     = (np.arange(n) + 0.5) * (size + gap) - offset
    return (xy, *np.meshgrid(xy, xy, indexing="ij"))

def sipm_extent(size, gap, nperside):
    """
    Compute the size of the sipm plane to scale the imshow plot.
    """
    offset = nperside * (size + gap) / 2.
    return (-offset, offset) * 2

def npmap(f, it):
    """
    Map and produce an array with that.
    """
    return np.array(list(map(f, it)))

def wire_pos(nwires, pitch):
    """
    Generate wire position sequence from - to +.
    """
    oneside = (np.arange(nwires//2) + 0.5) * pitch
    return np.concatenate([-oneside[::-1], oneside])

def wire_limits(nwires, pitch):
    """
    Generate wire focusing binning, i.e. the position
    edges that determine the focalization to each wire.
    """
    p  = wire_pos(nwires, pitch)
    dp = np.diff(p)[0]
    return np.append(p - dp/2, p[-1] + dp/2)

def get_nwires(diam, pitch):
    pos    = np.zeros(2)
    nwires = 0
    while pos.max() <= diam/2:
        nwires += 2
        pos = wire_pos(nwires, pitch)
    return nwires - 2

def empty_sensor_df(nperside, plane):
    """
    An empty dataframe that contains all sensors with zero counts.
    """
    def output(offset):
        return pd.DataFrame(dict(counts=0), index=pd.Index(np.arange(nperside**2) + offset, name="sensor_id"))
    if   plane == "np": return output(0)
    elif plane == "fp": return pd.concat([output(1000*i) for i in range(1,5)])
    else              : raise ValueError(f"Invalid plane: {plane}. Options are 'np' or 'fp'.")

def image(df, nperside, plane):
    """
    Count sensor hits and generate an image.
    """
    df = df.set_index("sensor_id")
    df = pd.concat([df, empty_sensor_df(nperside, plane)])
    df = df.groupby(level=0).sum()

    n = nperside if plane == "np" else 2*nperside
    colname = f"image_{plane}"
    return pd.DataFrame({colname: [df.sort_index().counts.values.reshape(1, n, n)]})

def rotate(xy, a):
    """
    Rotate an array of `xy`s by an angle `a` conterclockwise.
    """
    c = np.cos(a)
    s = np.sin(a)
    m = np.array([ [c, -s]
                 , [s,  c] ])
    return m.dot(xy)

def gate_pos(df, nwires, pitch, angle, gate_distance):
    """
    Compute gate position based on the approach angle to the wire.
    """
    xrot, yrot = rotate(df.loc[:, list("xy")].values.T, angle)
    xi         = np.clip(np.digitize(xrot, wire_limits(nwires, pitch)) - 1, 0, nwires-1)
    wx         = wire_pos(nwires, pitch)[xi]
    dx         = xrot - wx
    a          = np.arctan(dx/(-df.z - gate_distance))
    dx         = a/(np.pi/2) * pitch/2
    x, y       = rotate(np.stack([wx+dx, yrot], axis=1).T, -angle)
    return pd.DataFrame(dict(event=df.event, x0=x, y0=y, z0=df.z))

def imgs_from_file(filename, signal, fp=False):
    """
    Read sources and sensor_hits.
    """
    signal = signal.lower()
    if signal not in "s1 s2".split():
        raise ValueError("Invalid signal. Must be 's1' or 's2'")
    config      = simconfig(filename)
    gate_dist   = config.d_gate_wire + config.mesh_thick
    nwires      = get_nwires(config.el_diam, config.thin_wire_pitch)
    nsipm       = config.n_sipm_side

    sources     = pd.read_hdf(filename, "/MC/sources").drop(columns="e n".split())
    if signal == "s1":
        sources = sources.rename(columns=dict(x="x0", y="y0", z="z0"))
    else:
        sources = gate_pos(sources, nwires, config.thin_wire_pitch, config.thin_wire_rot*np.pi/180, gate_dist)
    sources     = sources.set_index("event")
    sources.insert(3, "r0", (sources.x0**2 + sources.y0**2)**0.5)

    sensor_hits = pd.read_hdf(filename, "/MC/sensor_hits")

    images_np = (sensor_hits.loc[sensor_hits.sensor_id < 1000]
                            .groupby("event sensor_id".split())
                            .count()
                            .rename(columns=dict(time="counts"))
                            .reset_index()
                            .groupby("event")
                            .apply(image, nsipm, "np", include_groups=False)
                            .reset_index(level=1, drop=True)
                )
    images = [images_np]
    if fp:
        images_fp = (sensor_hits.loc[sensor_hits.sensor_id >= 1000]
                                .groupby("event sensor_id".split())
                                .count()
                                .rename(columns=dict(time="counts"))
                                .reset_index()
                                .groupby("event")
                                .apply(image, nsipm, "fp", include_groups=False)
                                .reset_index(level=1, drop=True)
                    )
        images.append(images_fp)

    out = pd.concat([sources, *images], axis=1)
    out.image_np     = out.image_np.transform(lambda a: a if isinstance(a, np.ndarray) else np.zeros((1,   nsipm,   nsipm)))
    if fp:
        out.image_fp = out.image_fp.transform(lambda a: a if isinstance(a, np.ndarray) else np.zeros((1, 2*nsipm, 2*nsipm)))
    return out

def simconfig(filename):
    df     = pd.read_hdf(filename, "/MC/config").set_index("name")
    config = AttrDict()
    for (key, value, _) in df.itertuples():
        value = str(value)
        if value=="true" : value = True
        if value=="false": value = False
        try:
            thetype = float if "." in value else int
            value = thetype(value)
        except:
            pass
        config[key] = value
    return config

def barycenter(df, sipm_x, sipm_y):
    img = df.image_np.values[0][0]
    tot = img.sum().sum()
    xb  = (sipm_x * img).sum().sum() / tot
    yb  = (sipm_y * img).sum().sum() / tot
    return pd.Series(dict(x=xb, y=yb))

def draw_el(ax, r_el):
    c = plt.Circle((0,0), r_el, facecolor="None", edgecolor="r", lw=4)
    ax.add_patch(c)
    ax.set_xlim(-r_el, r_el)
    ax.set_ylim(-r_el, r_el)

def gang_array(a, nx, ny):
    assert len(a.shape)  == 2, f"{a.shape}"
    assert a.shape[0]%nx == 0, f"{a.shape}"
    assert a.shape[1]%ny == 0, f"{a.shape}"
    a = (a.reshape(a.shape[0]//nx, nx, -1, ny)
          .swapaxes(1, 2)
          .sum(axis=-1)
          .sum(axis=-1))
    return a

def gang_image(df, nx, ny):
    img = df.image.iloc[0][0] # 2d array
    img = gang_array(img, nx, ny)
    return df.assign(image = [img[np.newaxis]])
