// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {
    LibParseOperand,
    BaseRainterpreterSubParserNPE2,
    Operand
} from "rain.interpreter/abstract/BaseRainterpreterSubParserNPE2.sol";
import {LibConvert} from "rain.lib.typecast/LibConvert.sol";
import {BadDynamicLength} from "rain.interpreter/error/ErrOpList.sol";
import {LibExternOpContextSenderNPE2} from "rain.interpreter/lib/extern/reference/op/LibExternOpContextSenderNPE2.sol";
import {LibExternOpContextCallingContractNPE2} from
    "rain.interpreter/lib/extern/reference/op/LibExternOpContextCallingContractNPE2.sol";

import {LibOrderBookSubParser, SUB_PARSER_WORD_PARSERS_LENGTH} from "../../lib/LibOrderBookSubParser.sol";

bytes constant SUB_PARSER_PARSE_META =
    hex"010000000000000200000000000000000000040000000000000000000000000000000109ac3000d3b4e8";
bytes constant SUB_PARSER_WORD_PARSERS = hex"045e047d";
bytes constant SUB_PARSER_OPERAND_HANDLERS = hex"051f051f";

contract OrderBookSubParser is BaseRainterpreterSubParserNPE2 {
    function subParserParseMeta() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_PARSE_META;
    }

    function subParserWordParsers() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_WORD_PARSERS;
    }

    function subParserOperandHandlers() internal pure virtual override returns (bytes memory) {
        return SUB_PARSER_OPERAND_HANDLERS;
    }

    function buildSubParserOperandHandlers() external pure returns (bytes memory) {
        unchecked {
            function(uint256[] memory) internal pure returns (Operand) handleOperandDisallowed = LibParseOperand
                .handleOperandDisallowed;
            uint256 handleOperandDisallowedPtr;
            assembly ("memory-safe") {
                handleOperandDisallowedPtr := handleOperandDisallowed
            }

            function(uint256[] memory) internal pure returns (Operand)[][] memory handlers;

            function(uint256[] memory) internal pure returns (Operand)[] contextBaseHandlers;
            assembly ("memory-safe") {
                contextBaseHandlers := mload(0x40)
                mstore(0x40, add(contextBaseHandlers, 0x60))
                mstore(contextBaseHandlers, 2)
            }



            function(uint256[] memory) internal pure returns (Operand) lengthPointer;
            uint256 length = SUB_PARSER_WORD_PARSERS_LENGTH;
            assembly ("memory-safe") {
                lengthPointer := length
            }
            function(uint256[] memory) internal pure returns (Operand)[SUB_PARSER_WORD_PARSERS_LENGTH + 1] memory
                handlersFixed = [
                    lengthPointer,
                    // order clearer
                    LibParseOperand.handleOperandDisallowed,
                    // orderbook
                    LibParseOperand.handleOperandDisallowed
                ];
            uint256[] memory handlersDynamic;
            assembly {
                handlersDynamic := handlersFixed
            }
            // Sanity check that the dynamic length is correct. Should be an
            // unreachable error.
            if (handlersDynamic.length != length) {
                revert BadDynamicLength(handlersDynamic.length, length);
            }
            return LibConvert.unsafeTo16BitBytes(handlersDynamic);
        }
    }

    function buildSubParserWordParsers() external pure returns (bytes memory) {
        unchecked {
            function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)
                lengthPointer;
            uint256 length = SUB_PARSER_WORD_PARSERS_LENGTH;
            assembly ("memory-safe") {
                lengthPointer := length
            }
            function(uint256, uint256, Operand) internal view returns (bool, bytes memory, uint256[] memory)[SUB_PARSER_WORD_PARSERS_LENGTH
                + 1] memory pointersFixed = [
                    lengthPointer,
                    LibExternOpContextSenderNPE2.subParser,
                    LibExternOpContextCallingContractNPE2.subParser
                ];
            uint256[] memory pointersDynamic;
            assembly {
                pointersDynamic := pointersFixed
            }
            // Sanity check that the dynamic length is correct. Should be an
            // unreachable error.
            if (pointersDynamic.length != length) {
                revert BadDynamicLength(pointersDynamic.length, length);
            }
            return LibConvert.unsafeTo16BitBytes(pointersDynamic);
        }
    }
}
