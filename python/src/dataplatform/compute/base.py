"""
Compute Protocol.
"""

from abc import ABC, abstractmethod, abstractproperty
from typing import (
    Callable,
    Generic,
    Mapping,
    Optional,
    Protocol,
    TypeVar,
    overload,
    runtime_checkable,
)

from dataplatform.compute.schema import SchemaProtocol
from dataplatform.storage.protocol import SupportsStorage

Compute_T = TypeVar("Compute_T")
U = TypeVar("U")
NameKey = TypeVar("NameKey", bound=str)


@runtime_checkable
class ComputeProtocol(Protocol, Generic[Compute_T]):
    name: str
    sink: SupportsStorage
    dependencies: Mapping[str, "ComputeProtocol"]
    schema: SchemaProtocol
    _compute_type: str
    _config: Optional[Compute_T]

    def execute(self, function: Callable[..., Compute_T]) -> Compute_T: ...

    def adjacency_matrix(self) -> list[tuple[str, str]]: ...

    @property
    def compute_type(self):
        return self._compute_type


class BaseCompute(ABC, Generic[Compute_T, U]):
    def __init__(
        self,
        name: str,
        sink: SupportsStorage,
        dependencies: Mapping[str, ComputeProtocol],
        schema: SchemaProtocol,
        compute_type: str,
        config: Optional[Compute_T] = None,
    ) -> None:
        self.name = name
        self.dependencies = dependencies
        self.sink = sink
        self.schema = schema
        self._config = config
        self._compute_type = compute_type

    @abstractproperty
    def input_data(self):
        raise NotImplementedError

    def load(self): ...

    @overload
    @abstractmethod
    def execute(self, function: None) -> U: ...

    @overload
    @abstractmethod
    def execute(self, function: Callable[..., U]) -> U: ...

    @abstractmethod
    def execute(self, function) -> U: ...

    def adjacency_matrix(self) -> list[tuple[str, str]]:
        adjency = []
        for _, dependency in self.dependencies.items():
            adjency.append((dependency.name, self.name))
        return adjency

    @property
    def config(self):
        return self._config

    @property
    def compute_type(self):
        return self._compute_type
