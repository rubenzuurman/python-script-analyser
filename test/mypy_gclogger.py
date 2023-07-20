from __future__ import annotations

import gc
import time
from typing import Mapping

# Random global variables (not in the original file, just for testing purposes).
GLOB_NAME = "Bananas are pretty good"
GLOB_PARAMETER = 100 ** 2
GLOB_OBJ = time.time()

# Random function (also not in the original file).
def random_function(param1, p2, p3, p4, p5=3, p6 = 78.5,    p7  =    [  (5,  6)   ,   ( 94 , 45 ) , "Banana Shrine"] ):
    print("hihi")
    for i in range(10):
        if i % 2 == 0:
            print(f"number {i}")
            if i % 3 == 0:
                print("Divisible by 6!")
        else:
            print("Do nothing")
            continue
    
    print("End of function")

class GcLogger:
    """Context manager to log GC stats and overall time."""

    def __enter__(self) -> GcLogger:
        self.gc_start_time: float | None = None
        self.gc_time = 0.0
        self.gc_calls = 0
        self.gc_collected = 0
        self.gc_uncollectable = 0
        gc.callbacks.append(self.gc_callback)
        self.start_time = time.time()
        return self

    def gc_callback(self, phase: str, info: Mapping[str, int]) -> None:
        if phase == "start":
            assert self.gc_start_time is None, "Start phase out of sequence"
            self.gc_start_time = time.time()
        elif phase == "stop":
            assert self.gc_start_time is not None, "Stop phase out of sequence"
            self.gc_calls += 1
            self.gc_time += time.time() - self.gc_start_time
            self.gc_start_time = None
            self.gc_collected += info["collected"]
            self.gc_uncollectable += info["uncollectable"]
        else:
            assert False, f"Unrecognized gc phase ({phase!r})"

    def __exit__(self, *args: object) -> None:
        while self.gc_callback in gc.callbacks:
            gc.callbacks.remove(self.gc_callback)

    def get_stats(self) -> Mapping[str, float]:
        end_time = time.time()
        result = {}
        result["gc_time"] = self.gc_time
        result["gc_calls"] = self.gc_calls
        result["gc_collected"] = self.gc_collected
        result["gc_uncollectable"] = self.gc_uncollectable
        result["build_time"] = end_time - self.start_time
        return result