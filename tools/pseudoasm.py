#!/usr/bin/env python3
import argparse
import struct
import sys
from pathlib import Path

IMAGE_MAGIC = b"BODE"

IMMEDIATE = 0x100
OPCODES = {
    "ADD": 1,
    "SUB": 2,
    "MULT": 3,
    "DIV": 4,
    "LOAD": 5,
    "STORE": 6,
    "BRANY": 7,
    "BRPOS": 8,
    "BRZERO": 9,
    "BRNEG": 10,
    "SYSCALL": 11,
}
ARITHMETIC = {"ADD", "SUB", "MULT", "DIV", "LOAD"}
BRANCHES = {"BRANY", "BRPOS", "BRZERO", "BRNEG"}


class AsmError(Exception):
    pass


def sections(lines):
    current = None
    for number, raw in enumerate(lines, start=1):
        line = raw.strip()
        lowered = line.lower()
        if lowered in (".code", ".data"):
            current = lowered
            continue
        if lowered in (".endcode", ".enddata"):
            current = None
            continue
        if current is None or not line or line.startswith("#"):
            continue
        yield current, number, line


def parse(text):
    instructions = []
    labels = {}
    pending_labels = []
    variables = {}

    for section, number, line in sections(text.splitlines()):
        tokens = line.split()
        if section == ".data":
            if len(tokens) < 2:
                raise AsmError(f"linha {number}: variável sem valor")
            name = tokens[0].lower()
            if name in variables:
                raise AsmError(f"linha {number}: variável '{tokens[0]}' repetida")
            variables[name] = (len(variables), int(tokens[1]))
            continue

        if tokens[0].endswith(":"):
            pending_labels.append((tokens[0][:-1].lower(), number))
            tokens = tokens[1:]
            if not tokens or tokens[0].startswith("#"):
                continue

        mnemonic = tokens[0].upper()
        if mnemonic not in OPCODES:
            raise AsmError(f"linha {number}: instrução '{tokens[0]}' desconhecida")
        if len(tokens) < 2:
            raise AsmError(f"linha {number}: '{mnemonic}' sem operando")

        for label, _ in pending_labels:
            labels[label] = len(instructions)
        pending_labels.clear()
        instructions.append((number, mnemonic, tokens[1]))

    if pending_labels:
        label, number = pending_labels[0]
        raise AsmError(f"linha {number}: label '{label}' sem instrução depois")

    return instructions, labels, variables


def encode_operand(number, mnemonic, operand, labels, variables):
    if mnemonic == "SYSCALL":
        return OPCODES[mnemonic], int(operand)

    if mnemonic in BRANCHES:
        target = labels.get(operand.lower())
        if target is None:
            raise AsmError(f"linha {number}: label '{operand}' não existe")
        return OPCODES[mnemonic], target

    if operand.startswith("#"):
        if mnemonic not in ARITHMETIC:
            raise AsmError(f"linha {number}: '{mnemonic}' não aceita modo imediato")
        return OPCODES[mnemonic] | IMMEDIATE, int(operand[1:])

    variable = variables.get(operand.lower())
    if variable is None:
        raise AsmError(f"linha {number}: variável '{operand}' não existe")
    index, _ = variable
    return OPCODES[mnemonic], index


def assemble(text):
    instructions, labels, variables = parse(text)

    code = bytearray()
    for number, mnemonic, operand in instructions:
        opcode, value = encode_operand(number, mnemonic, operand, labels, variables)
        code += struct.pack("<Ii", opcode, value)

    data = bytearray()
    for _, value in sorted(variables.values()):
        data += struct.pack("<i", value)

    header = struct.pack("<II", len(instructions), len(variables))
    return bytes(header + code + data)


def build_image(programs):
    image = bytearray(IMAGE_MAGIC)
    image += struct.pack("<I", len(programs))
    for program in programs:
        image += struct.pack("<I", len(program))
        image += program
    return bytes(image)


def main():
    parser = argparse.ArgumentParser(description="Monta programas do T1 numa imagem para a flash do bode_os")
    parser.add_argument("sources", nargs="+", type=Path)
    parser.add_argument("-o", "--output", type=Path, required=True, help="imagem bruta (.bin)")
    args = parser.parse_args()

    programs = []
    for source in args.sources:
        try:
            program = assemble(source.read_text())
        except (AsmError, ValueError) as error:
            sys.exit(f"{source}: {error}")
        programs.append(program)
        print(f"{source}: {len(program)} bytes")

    image = build_image(programs)
    args.output.write_bytes(image)
    print(f"{args.output}: {len(programs)} programas, {len(image)} bytes")


if __name__ == "__main__":
    main()
