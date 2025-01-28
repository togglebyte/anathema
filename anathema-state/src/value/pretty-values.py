import gdb

class PendingValuePrinter:
    "Print a PendingValue"

    types = [
        'corrupt type information',
        'Int',
        'Float',
        'Char',
        'String',
        'Bool',
        'Hex',
        'Map',
        'List',
        'Composite',
    ]

    def __init__(self, val):
        owned = val['__0']['__0']['__0']
        self.sub = val['__0']['__1']['__0']

        self.index = owned & 0xFFFFFFFF
        self.gen = (owned >> 32) & 0x00FF
        aux = owned >> 48
        self.tpe = self.types[aux] 

        if not self.tpe:
            self.tpe = self.types[0]

    def to_string(self):
        return "<PendingValue ({}) idx: {} | g: {} | sub: {}>".format(
            self.tpe,
            self.index, 
            self.gen, 
            self.sub
        )


def lookup(val):
    lookup_tag = val.type.tag

    if lookup_tag is None:
        return None

    if "anathema_state::value::PendingValue" == lookup_tag:
        return PendingValuePrinter(val)

    return None


gdb.current_objfile().pretty_printers.append(lookup)
