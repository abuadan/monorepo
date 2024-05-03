"""
"""
from typing import Protocol, TypeVar, runtime_checkable, Callable


T = TypeVar("T")
U = TypeVar("U")
NameKey = TypeVar("NameKey", bound=str)
Func = Callable[..., U]


@runtime_checkable
class SupportsStorage(Protocol):
    def write(self) -> object:
        ...

    def read(self) -> object:
        ...
