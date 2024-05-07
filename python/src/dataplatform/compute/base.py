"""
Compute Protocol.
"""
from abc import ABC, abstractmethod, abstractproperty
from typing import (
    Generic,
    Mapping,
    Optional,
    Protocol,
    TypeVar,
    runtime_checkable,
    Callable,
    overload,
)
from src.dataplatform.compute.schema import SchemaProtocol
from src.dataplatform.storage.protocol import SupportsStorage


T = TypeVar("T")
U = TypeVar("U")
NameKey = TypeVar("NameKey", bound=str)


@runtime_checkable
class ComputeProtocol(Protocol):
    name: str
    sink: SupportsStorage
    dependencies: Mapping[str, "ComputeProtocol"]
    schema: SchemaProtocol
    compute_type: str

    def execute(self, function: Callable[..., T]) -> T:
        ...


class BaseCompute(ABC, Generic[T, U]):
    def __init__(
        self,
        name: str,
        sink: SupportsStorage,
        dependencies: Mapping[NameKey, ComputeProtocol],
        schema: SchemaProtocol,
        compute_type: str,
        config: Optional[T] = None,
    ) -> None:
        self.name = name
        self.dependcies = dependencies
        self.sink = sink
        self.schema = schema
        self._config = config
        self._compute_type = compute_type

    @abstractproperty
    def input_data(self):
        raise NotImplementedError

    def load(self):
        ...

    @overload
    @abstractmethod
    def execute(self, function: None) -> U:
        ...

    @overload
    @abstractmethod
    def execute(self, function: Callable[..., U]) -> U:
        ...

    @abstractmethod
    def execute(self, function) -> U:
        ...

    @property
    def config(self):
        return self._config
