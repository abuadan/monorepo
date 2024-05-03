"""
Compute Protocol.
"""
from abc import ABC, abstractmethod, abstractproperty
from typing import Mapping, Protocol, TypeVar, runtime_checkable, Callable
from src.dataplatform.compute.schema import SchemaDataClass, SchemaProtocol

from src.dataplatform.storage.protocol import SupportsStorage


T = TypeVar("T")
U = TypeVar("U")
NameKey = TypeVar("NameKey", bound=str)


class SupportsDataClass(Protocol):
    def __dataclass__(self):
        ...


@runtime_checkable
class SupportsProcess(Protocol):
    output_sink: SupportsStorage
    dependencies: Mapping[NameKey, "SupportsProcess"]

    def process(self, function: Callable[..., T]) -> T:
        ...


class BaseCompute(ABC):
    def __init__(
        self,
        sink: SupportsStorage,
        dependencies: Mapping[NameKey, SupportsProcess],
        schema: SchemaProtocol,
    ) -> None:
        self.dependcies = dependencies
        self.sink = sink
        self.schema = schema

    @abstractproperty
    def input_data(self):
        raise NotImplementedError

    def load(self):
        ...

    @abstractmethod
    def process(self, function: Callable[..., T]) -> T:
        ...
