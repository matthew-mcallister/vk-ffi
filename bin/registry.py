"""This module parses the Vulkan XML registry for use by code generators
and the like.
"""
from __future__ import annotations

import copy
from os import name
import re
import typing as ty
from dataclasses import dataclass, field
from xml.etree.ElementTree import Element


ARRAY_SIZES = {
    'VK_MAX_PHYSICAL_DEVICE_NAME_SIZE': 256,
    'VK_UUID_SIZE':                     16,
    'VK_LUID_SIZE':                     8,
    "VK_LUID_SIZE_KHR":                 8,
    'VK_MAX_EXTENSION_NAME_SIZE':       256,
    'VK_MAX_DESCRIPTION_SIZE':          256,
    'VK_MAX_MEMORY_TYPES':              32,
    'VK_MAX_MEMORY_HEAPS':              16,
    'VK_MAX_DEVICE_GROUP_SIZE':         32,
    'VK_MAX_DEVICE_GROUP_SIZE_KHR':     32,
    'VK_MAX_DRIVER_NAME_SIZE':          256,
    'VK_MAX_DRIVER_INFO_SIZE':          256,
    'VK_MAX_DRIVER_NAME_SIZE_KHR':      256,
    'VK_MAX_DRIVER_INFO_SIZE_KHR':      256,
    'VK_SHADER_UNUSED_KHR':             0xFFFFFFFF,
    'VK_SHADER_UNUSED_NV':              0xFFFFFFFF,
    'VK_MAX_GLOBAL_PRIORITY_SIZE_KHR':  16,
    'VK_MAX_GLOBAL_PRIORITY_SIZE_EXT':  16,
    'VK_MAX_SHADER_MODULE_IDENTIFIER_SIZE_EXT': 32,
}


def array_len(len):
    try:
        return int(len)
    except ValueError:
        return ARRAY_SIZES[len]


# Here be C PARSING STUFF

# FYI, these parsers give bogus results if there are any syntax errors


class TokenStream:
    tokens: list[str]

    def __init__(self, tokens):
        self.tokens = tokens

    def __len__(self):
        return len(self.tokens)

    def __bool__(self):
        return bool(self.tokens)

    def __iter__(self):
        return iter(self.tokens)

    def lookahead(self, n):
        return self.tokens[:min(n, len(self))]

    def pop(self):
        res = self.tokens[0]
        self.tokens = self.tokens[1:]
        return res

    def take(self, n: int):
        res = self.tokens[:n]
        self.tokens = self.tokens[n:]
        return res

    def expect(self, expected: str | list[str]):
        """Removes the given prefix from the stream and raises an error
        if the prefix is not found."""
        if isinstance(expected, str):
            expected = [expected]
        found = self.lookahead(len(expected))
        if found != expected:
            raise ValueError(f'expected: {expected}, found: {found}')
        else:
            self.take(len(expected))

    def accept(self, accepted: str | list[str]):
        """Removes the given prefix from the stream if found."""
        if isinstance(accepted, str):
            accepted = [accepted]
        if self.lookahead(len(accepted)) == accepted:
            self.take(len(accepted))
            return True
        return False


TOKENIZE_REGEX = re.compile(r'\w+|\S')


def tokenize_c(src: str) -> TokenStream:
    """Breaks up a string of C code into its component tokens.

    Not very accurately of course; it only matches ASCII identifiers and
    punctuation.
    """
    return TokenStream(TOKENIZE_REGEX.findall(src))


def c_tokens(source: str):
    if isinstance(source, str):
        return tokenize_c(source)
    else:
        assert isinstance(source, TokenStream)
        return source


def parse_c_basic_type(source):
    """Parses a C type that includes a type or struct name and optional
    pointer and `const` qualifiers.

    >>> parse_c_basic_type('const struct Foo**')
    """
    tokens = c_tokens(source)
    base = None
    qualifiers = []
    idx = 0
    for idx, token in enumerate(tokens):
        if token in ('const', '*'):
            qualifiers.append(token)
        elif token == 'struct':
            pass
        elif not base:
            base = token
        else:
            break
    assert name
    tokens.take(idx)
    return TypeExpr(Name.from_ident(base), qualifiers)


def parse_c_decl(source):
    """Parses a basic C typed variable or aggregate member declaration.

    Can handle array and array-of-pointers syntax, but not
    pointer-to-array syntax. Array length must be a single int or
    identifier token; empty length and expressions don't work.
    """
    tokens = c_tokens(source)
    ty = parse_c_basic_type(tokens)
    name = tokens.pop()
    if tokens.accept('['):
        len = array_len(tokens.pop())
        ty.len = len
        tokens.expect(']')
    return Decl(name, ty)


def parse_c_func_pointer(source):
    tokens = c_tokens(source)
    ret = parse_c_basic_type(tokens)
    tokens.expect(['(', 'VKAPI_PTR', '*'])
    fp_name = tokens.pop()
    tokens.expect([')', '('])

    args = []
    while True:
        ty = parse_c_basic_type(tokens)
        name = None

        next = tokens.pop()
        if next not in '),':
            name = next
            next = tokens.pop()
            assert next in '),'

        args.append(Decl(name, ty))

        if next == ')':
            break

    return FuncPointer(Name.from_ident(fp_name), ret, args)


# Here be REGISTRY STUFF


def strip_prefix(prefix, name):
    if name.startswith(prefix):
        return name[len(prefix):]
    else:
        return name


def strip_suffix(name, suffix):
    if name.endswith(suffix):
        return name[:-len(suffix)]
    else:
        return name


TITLE_WORDS_REGEX = re.compile(r'(?<=[a-z])[A-Z]')


def title_to_all_caps(title):
    """Converts a title-case string to all caps."""
    return TITLE_WORDS_REGEX.sub(lambda s: '_' + s.group(0), title).upper()


@dataclass(frozen=True)
class Name:
    namespace: str
    base: str

    def __str__(self):
        return self.namespace + self.base

    PREFIX_REGEX = re.compile('^(?:vk|Vk|VK_|PFN_vk)')

    @staticmethod
    def from_ident(ident: str) -> Name:
        namespace = ''
        base = ''
        if m := Name.PREFIX_REGEX.match(ident):
            namespace = ident[:m.end()]
            base = ident[m.end():]
        else:
            base = ident
        return Name(namespace=namespace, base=base)


@dataclass
class TypeExpr:
    """A simple C type expression.

    Qualifiers are sorted from innermost to outermost."""
    base: Name
    qualifiers: ty.List[str]
    # TODO: Need multiple dimensions for matrices
    len: ty.Optional[int] = field(default=None)


@dataclass
class Decl:
    name: str
    ty: TypeExpr


@dataclass
class AggregateMember(Decl):
    values: ty.List[str]


@dataclass
class Arg:
    name: ty.Optional[str]
    ty: TypeExpr


@dataclass
class Func:
    name: Name
    ret: ty.Optional[TypeExpr]
    args: ty.List[ty.Union[Arg, Decl]]


@dataclass
class EnumMember:
    name: Name
    value: int


VENDOR_TAGS = []


def strip_vendor_suffix(name):
    for tag in VENDOR_TAGS:
        if name.endswith(tag):
            res = name[:-len(tag)]
            if res.endswith('_'):
                return res[:-1]
            else:
                return res
    return name


@dataclass
class Enum:
    name: Name
    ty: str
    members: ty.List[EnumMember] = field(default_factory=list)

    def __post_init__(self):
        assert self.ty in ('bitmask', 'bitmask64', 'enum')
        self.prefix = self.get_prefix()

    def get_prefix(self):
        prefix = strip_vendor_suffix(str(self.name))
        prefix = strip_suffix(prefix, 'FlagBits')
        prefix = title_to_all_caps(prefix)
        return prefix + '_'

    def add_member(self, ident: str, value: int):
        for member in self.members:
            if ident == str(member.name):
                assert value == member.value
                return

        if ident.startswith(self.prefix):
            base = ident[len(self.prefix):]
            name = Name(namespace=self.prefix, base=base)
        else:
            name = Name.from_ident(ident)
        self.members.append(EnumMember(name, value))


@dataclass
class Alias:
    name: Name
    target: Name


@dataclass
class Extern:
    name: str
    header: ty.Optional[str] = None

    def __post_init__(self):
        assert not self.header or self.header.endswith('.h')


def datatype(category):
    def impl_category(self):
        return category

    def inner(cls):
        cls.category = impl_category
        return cls
    return inner


@datatype(category='type_alias')
@dataclass
class TypeAlias:
    name: Name
    target: Name


@datatype(category='handle')
@dataclass
class Handle:
    name: Name
    parents: ty.List[Name]
    dispatchable: bool
    level: ty.Optional[str] = field(default=None)

    def __post_init__(self):
        if self.name.base in ('Instance', 'Device'):
            self.level = self.name.base


@datatype(category='aggregate')
@dataclass
class Aggregate:
    name: Name
    members: ty.List[AggregateMember]
    ty: str


@datatype(category='func_pointer')
class FuncPointer(Func):
    pass


Command = Func


@dataclass
class Extension:
    name: str
    level: str


# Stuff that's easier to hardcode than to parse

EXTERNS = [
    Extern('ANativeWindow'),
    Extern('AHardwareBuffer'),
    Extern('CAMetalLayer'),
    Extern('IOSurface'),
    Extern('wl_display'),
    Extern('wl_surface'),
    Extern('IDirectFB'),
    Extern('IDirectFBSurface'),
    Extern('_screen_context'),
    Extern('_screen_window'),
]


def typedef(name: str, value: str) -> TypeAlias:
    return TypeAlias(Name.from_ident(name), Name('', value))


ALIASES = [
    typedef('MTLDevice_id', '*mut c_void'),
    typedef('MTLCommandQueue_id', '*mut c_void'),
    typedef('MTLBuffer_id', '*mut c_void'),
    typedef('MTLTexture_id', '*mut c_void'),
    typedef('MTLSharedEvent_id', '*mut c_void'),
    typedef('IOSurfaceRef', '*mut IOSurface'),
    typedef('Display', 'u32'),
    typedef('VisualID', 'u32'),
    typedef('Window', 'u32'),
    typedef('RROutput', 'c_ulong'),
    typedef('DWORD', 'c_int'),
    typedef('HANDLE', 'c_int'),
    typedef('HINSTANCE', 'c_int'),
    typedef('HMONITOR', 'c_int'),
    typedef('HWND', 'c_int'),
    typedef('LPCWSTR', 'c_int'),
    typedef('SECURITY_ATTRIBUTES', 'c_int'),
    typedef('xcb_connection_t', 'c_int'),
    typedef('xcb_window_t', 'c_int'),
    typedef('xcb_visualid_t', 'c_int'),
    typedef('zx_handle_t', 'c_int'),
    typedef('GgpStreamDescriptor', 'c_int'),
    typedef('GgpFrameToken', 'c_int'),
]


def remove_comments(elem):
    for child in elem:
        if child.tag == 'comment':
            elem.remove(child)
        else:
            remove_comments(child)


def elem_txt(elem):
    return ' '.join(elem.itertext())


def resolve_aliases(entry):
    enum = entry['enum']
    for name, target in entry['aliases']:
        for member in enum.members:
            if str(member.name) == target:
                enum.add_member(name, member.value)
                break
        else:
            raise ValueError(f'No member {name} of {enum.name}')


def fill_handle_levels(types):
    handles = {
        ty.name.base: ty
        for ty in types
        if ty.category() == 'handle'
    }
    for handle in handles.values():
        if handle.level or not handle.parents:
            continue
        ancestor = handles[handle.parents[0].base]
        while True:
            if ancestor.level:
                handle.level = ancestor.level
                break
            else:
                ancestor = handles[ancestor.parents[0].base]


class Registry:
    def __init__(self, **kwargs):
        self.builtins = []
        self.enums = {}
        self.externs = copy.deepcopy(EXTERNS)
        self.types = copy.deepcopy(ALIASES)
        self.commands = []
        self.extensions = []

    def parse_registry(self, root):
        # N.B. this modifies input
        remove_comments(root)

        global VENDOR_TAGS
        VENDOR_TAGS = []

        for elem in root:
            if elem.tag == 'tags':
                for child in elem:
                    VENDOR_TAGS.append(child.attrib['name'])
            elif elem.tag == 'types':
                for child in elem:
                    self.parse_type(child)
            elif elem.tag == 'enums':
                self.parse_enums(elem)
            elif elem.tag == 'commands':
                for child in elem:
                    self.parse_command(child)
            elif elem.tag == 'feature':
                self.parse_feature(elem)
            elif elem.tag == 'extensions':
                self.parse_extensions(elem)

        for enum in self.enums.values():
            resolve_aliases(enum)
        self.enums = [elem['enum'] for elem in self.enums.values()]

        fill_handle_levels(self.types)

    def parse_enums(self, elem):
        """Parses enum member definitions."""
        raw_name = elem.attrib['name']
        if raw_name == 'API Constants':
            return
        try:
            entry = self.enums[raw_name]
        except:
            entry = self.add_enum_stub(raw_name)

        enum = entry['enum']
        enum.ty = elem.attrib['type']
        if elem.attrib.get('bitwidth') == '64':
            enum.ty = 'bitmask64'

        for child in elem.findall('enum'):
            name = child.attrib['name']
            try:
                entry['aliases'].append((name, child.attrib['alias']))
                continue
            except KeyError:
                pass

            try:
                value = int(child.attrib['value'], 0)
            except KeyError:
                bitpos = int(child.attrib['bitpos'], 0)
                value = 1 << bitpos

            enum.add_member(name, value)

    def parse_type(self, elem: Element) -> None:
        if elem.tag == 'comment':
            return
        assert elem.tag == 'type'

        try:
            target = Name.from_ident(elem.attrib['alias'])
            name = Name.from_ident(elem.attrib['name'])
            self.types.append(TypeAlias(name, target))
            return
        except KeyError:
            pass

        try:
            category = elem.attrib['category']
        except KeyError:
            self.parse_opaque_type(elem)
            return

        if category == 'basetype':
            name = Name.from_ident(elem.find('name').text)
            if (ty := elem.find('type')) is not None:
                target = Name.from_ident(ty.text)
                self.types.append(TypeAlias(name, target))
            else:
                # Remaining cases are hardcoded since the Vulkan
                # committee mostly just pastes raw C code
                # TODO: Add to list of missing types if not hardcoded
                pass
        elif category == 'enum':
            self.parse_enum(elem)
        elif category == 'bitmask':
            self.parse_bitmask(elem)
        elif category == 'handle':
            self.parse_handle(elem)
        elif category == 'funcpointer':
            tokens = tokenize_c(elem_txt(elem))
            tokens.expect('typedef')
            self.types.append(parse_c_func_pointer(tokens))
        elif category in ('struct', 'union'):
            self.parse_aggregate(elem)

    def add_enum_stub(self, raw_name):
        """Defines a memberless enum that may get filled in later."""
        assert raw_name not in self.enums
        name = Name.from_ident(raw_name)
        entry = {
            'enum': Enum(name=name, ty='enum'),
            'aliases': [],
        }
        self.enums[raw_name] = entry
        return entry

    def parse_enum(self, elem):
        """Parses an enum type declaration.

        For parsing of member definitions, see the poorly named
        parse_enums method.
        """
        self.add_enum_stub(elem.attrib['name'])

    def parse_bitmask(self, elem):
        name = elem.find('name').text
        enum_name = Name.from_ident(name)
        try:
            # Alias an enum defined elsewhere
            target = Name.from_ident(elem.attrib['requires'])
            self.types.append(TypeAlias(enum_name, target))
        except KeyError:
            # Define a memberless placeholder enum
            if elem.find('type').text == 'VkFlags64':
                ty = 'bitmask64'
            else:
                ty = 'bitmask'
            self.enums[name] = {
                'enum': Enum(name=enum_name, ty=ty),
                'aliases': [],
            }

    def parse_handle(self, elem):
        name = Name.from_ident(elem.find('name').text)
        parents = elem.get('parent')
        parents = parents.split(',') if parents else []
        parents = [Name.from_ident(p) for p in parents]

        ty = elem.find('type').text
        dispatchable = ty == 'VK_DEFINE_HANDLE'
        if not dispatchable:
            assert ty == 'VK_DEFINE_NON_DISPATCHABLE_HANDLE'

        self.types.append(Handle(name, parents, dispatchable))

    def parse_aggregate(self, elem):
        name = Name.from_ident(elem.attrib['name'])
        members = []
        for child in elem:
            decl = parse_c_decl(elem_txt(child))
            try:
                values = child.attrib['values'].split(',')
            except KeyError:
                values = []
            members.append(AggregateMember(
                name=decl.name,
                ty=decl.ty,
                values=values,
            ))
        category = elem.attrib['category']
        self.types.append(Aggregate(name, members, category))

    def parse_opaque_type(self, elem):
        name = elem.attrib['name']
        if (requires := elem.get('requires')) and requires != 'vk_platform':
            self.externs.append(Extern(name, header=requires))
        else:
            self.builtins.append(name)

    def parse_command(self, elem):
        try:
            target = Name.from_ident(elem.attrib['alias'])
            name = Name.from_ident(elem.attrib['name'])
            return Alias(name, target)
        except KeyError:
            pass

        tokens = c_tokens(elem_txt(elem.find('proto')))
        ret = parse_c_basic_type(tokens)
        name = Name.from_ident(tokens.pop())
        args = [parse_c_decl(elem_txt(p)) for p in elem.findall('param')]

        command = Command(name=name, ret=ret, args=args)
        self.commands.append(command)

    def parse_feature(self, elem):
        for child in elem.findall('./require/enum'):
            self.parse_enum_ext(0, child)

    def parse_extensions(self, elem):
        for ext in elem:
            self.parse_extension(ext)
            base_extnumber = ext.attrib['number']
            for child in ext.findall('./require/enum'):
                self.parse_enum_ext(base_extnumber, child)

    def parse_extension(self, elem):
        if elem.get('supported') == 'vulkan':
            name = elem.attrib['name']
            level = elem.attrib['type']
            self.extensions.append(Extension(name, level))

    def parse_enum_ext(self, base_extnumber, elem):
        try:
            extends = elem.attrib['extends']
        except KeyError:
            return

        entry = self.enums[extends]
        enum = entry['enum']
        name = elem.attrib['name']

        try:
            entry['aliases'].append((name, elem.attrib['alias']))
            return
        except KeyError:
            pass

        try:
            bitpos = elem.attrib['bitpos']
            value = 1 << int(bitpos)
            enum.add_member(name, value)
            return
        except KeyError:
            pass

        try:
            value = int(elem.attrib['value'], 0)
            enum.add_member(name, value)
            return
        except KeyError:
            pass

        extnumber = int(elem.get('extnumber', base_extnumber), 0)
        offset = int(elem.attrib['offset'], 0)

        sign = +1
        if elem.get('dir') == '-':
            sign = -1

        EXT_BASE = 1000000000
        EXT_BLOCK_SIZE = 1000

        # This formula comes from `generatory.py` in the Vulkan registry repo
        value = EXT_BASE + (extnumber - 1) * EXT_BLOCK_SIZE + offset
        value *= sign

        enum.add_member(name, value)

    def tree(self):
        """Returns a serialization-friendly version of the registry."""
        return {
            'builtins': self.builtins,
            'externs': self.externs,
            'enums': self.enums,
            'types': [
                {'category': ty.category(), 'type': ty}
                for ty in self.types
            ],
            'commands': self.commands,
            'extensions': self.extensions,
        }
