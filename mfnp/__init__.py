try:
    from getconfig import *
    from run import *
except ModuleNotFoundError:
    from mfnp.getconfig import *
    from mfnp.run import *

__version__ = "1.0"