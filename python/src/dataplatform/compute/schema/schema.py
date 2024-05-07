from typing import (
    Any,
    Callable,
    ClassVar,
    Generic,
    Protocol,
    TypeVar,
    runtime_checkable,
)


from pydantic.fields import FieldInfo


class StandardDataclassProtocol(Protocol):
    """Standard Dataclass protocol"""

    __dataclass_fields__: ClassVar[dict[str, Any]]
    __dataclass_params__: ClassVar[Any]  # in reality `dataclasses._DataclassParams`
    __post_init__: ClassVar[Callable[..., None]]

    def __init__(self, *args: object, **kwargs: object) -> None:
        pass


class PydanticDataclassProtocol(StandardDataclassProtocol, Protocol):
    """A protocol containing attributes only available once a class has been decorated as a Pydantic dataclass.

    Attributes:
        __pydantic_fields__: Metadata about the fields defined on the dataclass.
    """

    __pydantic_fields__: ClassVar[dict[str, FieldInfo]]


SchemaDataClass = StandardDataclassProtocol | PydanticDataclassProtocol
_SchemaProtocol = TypeVar("_SchemaProtocol", bound=SchemaDataClass)


@runtime_checkable
class SchemaProtocol(Generic[_SchemaProtocol]):
    pass
