/**
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

lexer grammar LangLexer;

@lexer::members {
  public boolean is_at(int pos) {
    return _input.index() == pos;
  }
}


tokens {
    EQ, LT, LE, EQEQ, NE, GE, GT, ANDAND, OROR, NOT, TILDE, PLUS,
    MINUS, STAR, SLASH, PERCENT, CARET, AND, OR, SHL, SHR, BINOP,
    BINOPEQ, LARROW, AT, DOT, DOTDOT, DOTDOTDOT, COMMA, SEMI, COLON,
    MOD_SEP, RARROW, FAT_ARROW, LPAREN, RPAREN, LBRACKET, RBRACKET,
    LBRACE, RBRACE, POUND, DOLLAR, UNDERSCORE, LIT_CHAR, LIT_BYTE,
    LIT_INTEGER, LIT_FLOAT, LIT_STR, LIT_STR_RAW, LIT_BYTE_STR,
    LIT_BYTE_STR_RAW, QUESTION, IDENT, LIFETIME, WHITESPACE, DOC_COMMENT,
    COMMENT, SHEBANG, UTF8_BOM
}

/*
===============================================================================
  Tokens for Case Insensitive Keywords
===============================================================================
*/
fragment A
	:	'A' | 'a';

fragment B
	:	'B' | 'b';

fragment C
	:	'C' | 'c';

fragment D
	:	'D' | 'd';

fragment E
	:	'E' | 'e';

fragment F
	:	'F' | 'f';

fragment G
	:	'G' | 'g';

fragment H
	:	'H' | 'h';

fragment I
	:	'I' | 'i';

fragment J
	:	'J' | 'j';

fragment K
	:	'K' | 'k';

fragment L
	:	'L' | 'l';

fragment M
	:	'M' | 'm';

fragment N
	:	'N' | 'n';

fragment O
	:	'O' | 'o';

fragment P
	:	'P' | 'p';

fragment Q
	:	'Q' | 'q';

fragment R
	:	'R' | 'r';

fragment S
	:	'S' | 's';

fragment T
	:	'T' | 't';

fragment U
	:	'U' | 'u';

fragment V
	:	'V' | 'v';

fragment W
	:	'W' | 'w';

fragment X
	:	'X' | 'x';

fragment Y
	:	'Y' | 'y';

fragment Z
	:	'Z' | 'z';


/*
===============================================================================
  Reserved Keywords
===============================================================================
*/

IMPORT: I M P O R T;
MOD: M O D;
PUB: P U B;
SELF: S E L F;
TYPE: T Y P E;

/* Expression-operator symbols */

EQ      : '=' ;
LT      : '<' ;
LE      : '<=' ;
EQEQ    : '==' ;
NE      : '!=' ;
GE      : '>=' ;
GT      : '>' ;
ANDAND  : '&&' ;
OROR    : '||' ;
NOT     : '!' ;
TILDE   : '~' ;
PLUS    : '+' ;
MINUS   : '-' ;
STAR    : '*' ;
SLASH   : '/' ;
PERCENT : '%' ;
CARET   : '^' ;
AND     : '&' ;
OR      : '|' ;
SHL     : '<<' ;
SHR     : '>>' ;
LARROW  : '<-' ;

BINOP
    : PLUS
    | SLASH
    | MINUS
    | STAR
    | PERCENT
    | CARET
    | AND
    | OR
    | SHL
    | SHR
    | LARROW
    ;

BINOPEQ : BINOP EQ ;

/* "Structural symbols" */

AT         : '@' ;
DOT        : '.' ;
DOTDOT     : '..' ;
DOTDOTDOT  : '...' ;
COMMA      : ',' ;
SEMI       : ';' ;
COLON      : ':' ;
MOD_SEP    : '::' ;
RARROW     : '->' ;
FAT_ARROW  : '=>' ;
LPAREN     : '(' ;
RPAREN     : ')' ;
LBRACKET   : '[' ;
RBRACKET   : ']' ;
LBRACE     : '{' ;
RBRACE     : '}' ;
POUND      : '#';
DOLLAR     : '$' ;
UNDERSCORE : '_' ;

// Literals

fragment HEXIT
  : [0-9a-fA-F]
  ;

fragment CHAR_ESCAPE
  : [nrt\\'"0]
  | [xX] HEXIT HEXIT
  | 'u' HEXIT HEXIT HEXIT HEXIT
  | 'U' HEXIT HEXIT HEXIT HEXIT HEXIT HEXIT HEXIT HEXIT
  | 'u{' HEXIT '}'
  | 'u{' HEXIT HEXIT '}'
  | 'u{' HEXIT HEXIT HEXIT '}'
  | 'u{' HEXIT HEXIT HEXIT HEXIT '}'
  | 'u{' HEXIT HEXIT HEXIT HEXIT HEXIT '}'
  | 'u{' HEXIT HEXIT HEXIT HEXIT HEXIT HEXIT '}'
  ;
fragment SUFFIX
  : IDENT
  ;
fragment INTEGER_SUFFIX
  : { _input.LA(1) != 'e' && _input.LA(1) != 'E' }? SUFFIX
  ;
LIT_CHAR
  : '\'' ( '\\' CHAR_ESCAPE
         | ~[\\'\n\t\r]
         | '\ud800' .. '\udbff' '\udc00' .. '\udfff'
         )
    '\'' SUFFIX?
  ;

LIT_BYTE
  : 'b\'' ( '\\' ( [xX] HEXIT HEXIT
                 | [nrt\\'"0] )
          | ~[\\'\n\t\r] '\udc00'..'\udfff'?
          )
    '\'' SUFFIX?
  ;

LIT_INTEGER

  : [0-9][0-9_]* INTEGER_SUFFIX?
  | '0b' [01_]+ INTEGER_SUFFIX?
  | '0o' [0-7_]+ INTEGER_SUFFIX?
  | '0x' [0-9a-fA-F_]+ INTEGER_SUFFIX?
  ;

LIT_FLOAT
  : [0-9][0-9_]* ('.' {
        /* dot followed by another dot is a range, not a float */
        _input.LA(1) != '.' &&
        /* dot followed by an identifier is an integer with a function call, not a float */
        _input.LA(1) != '_' &&
        !(_input.LA(1) >= 'a' && _input.LA(1) <= 'z') &&
        !(_input.LA(1) >= 'A' && _input.LA(1) <= 'Z')
  }? | ('.' [0-9][0-9_]*)? ([eE] [-+]? [0-9][0-9_]*)? SUFFIX?)
  ;

LIT_STR
  : '"' ('\\\n' | '\\\r\n' | '\\' CHAR_ESCAPE | .)*? '"' SUFFIX?
  ;

LIT_BYTE_STR : 'b' LIT_STR ;
LIT_BYTE_STR_RAW : 'b' LIT_STR_RAW ;

/* this is a bit messy */

fragment LIT_STR_RAW_INNER
  : '"' .*? '"'
  | LIT_STR_RAW_INNER2
  ;

fragment LIT_STR_RAW_INNER2
  : POUND LIT_STR_RAW_INNER POUND
  ;

LIT_STR_RAW
  : 'r' LIT_STR_RAW_INNER SUFFIX?
  ;


QUESTION : '?';

IDENT : XID_Start XID_Continue* ;

fragment QUESTION_IDENTIFIER : QUESTION? IDENT;

LIFETIME : '\'' IDENT ;

WHITESPACE : [ \r\n\t]+ -> skip;

UNDOC_COMMENT     : '////' ~[\n]* -> type(COMMENT) ;
YESDOC_COMMENT    : '///' ~[\r\n]* -> type(DOC_COMMENT) ;
OUTER_DOC_COMMENT : '//!' ~[\r\n]* -> type(DOC_COMMENT) ;
LINE_COMMENT      : '//' ( ~[/\n] ~[\n]* )? -> type(COMMENT) ;

DOC_BLOCK_COMMENT
  : ('/**' ~[*] | '/*!') (DOC_BLOCK_COMMENT | .)*? '*/' -> type(DOC_COMMENT)
  ;

BLOCK_COMMENT : '/*' (BLOCK_COMMENT | .)*? '*/' -> type(COMMENT) ;

/* these appear at the beginning of a file */

SHEBANG : '#!' { is_at(2) && _input.LA(1) != '[' }? ~[\r\n]* -> type(SHEBANG) ;

UTF8_BOM : '\ufeff' { is_at(1) }? -> skip ;

fragment XID_Start :
      '\u0041' .. '\u005a'
    | '_'
    | '\u0061' .. '\u007a'
    | '\u00aa'
    | '\u00b5'
    | '\u00ba'
    | '\u00c0' .. '\u00d6'
    | '\u00d8' .. '\u00f6'
    | '\u00f8' .. '\u0236'
    | '\u0250' .. '\u02c1'
    | '\u02c6' .. '\u02d1'
    | '\u02e0' .. '\u02e4'
    | '\u02ee'
    | '\u0386'
    | '\u0388' .. '\u038a'
    | '\u038c'
    | '\u038e' .. '\u03a1'
    | '\u03a3' .. '\u03ce'
    | '\u03d0' .. '\u03f5'
    | '\u03f7' .. '\u03fb'
    | '\u0400' .. '\u0481'
    | '\u048a' .. '\u04ce'
    | '\u04d0' .. '\u04f5'
    | '\u04f8' .. '\u04f9'
    | '\u0500' .. '\u050f'
    | '\u0531' .. '\u0556'
    | '\u0559'
    | '\u0561' .. '\u0587'
    | '\u05d0' .. '\u05ea'
    | '\u05f0' .. '\u05f2'
    | '\u0621' .. '\u063a'
    | '\u0640' .. '\u064a'
    | '\u066e' .. '\u066f'
    | '\u0671' .. '\u06d3'
    | '\u06d5'
    | '\u06e5' .. '\u06e6'
    | '\u06ee' .. '\u06ef'
    | '\u06fa' .. '\u06fc'
    | '\u06ff'
    | '\u0710'
    | '\u0712' .. '\u072f'
    | '\u074d' .. '\u074f'
    | '\u0780' .. '\u07a5'
    | '\u07b1'
    | '\u0904' .. '\u0939'
    | '\u093d'
    | '\u0950'
    | '\u0958' .. '\u0961'
    | '\u0985' .. '\u098c'
    | '\u098f' .. '\u0990'
    | '\u0993' .. '\u09a8'
    | '\u09aa' .. '\u09b0'
    | '\u09b2'
    | '\u09b6' .. '\u09b9'
    | '\u09bd'
    | '\u09dc' .. '\u09dd'
    | '\u09df' .. '\u09e1'
    | '\u09f0' .. '\u09f1'
    | '\u0a05' .. '\u0a0a'
    | '\u0a0f' .. '\u0a10'
    | '\u0a13' .. '\u0a28'
    | '\u0a2a' .. '\u0a30'
    | '\u0a32' .. '\u0a33'
    | '\u0a35' .. '\u0a36'
    | '\u0a38' .. '\u0a39'
    | '\u0a59' .. '\u0a5c'
    | '\u0a5e'
    | '\u0a72' .. '\u0a74'
    | '\u0a85' .. '\u0a8d'
    | '\u0a8f' .. '\u0a91'
    | '\u0a93' .. '\u0aa8'
    | '\u0aaa' .. '\u0ab0'
    | '\u0ab2' .. '\u0ab3'
    | '\u0ab5' .. '\u0ab9'
    | '\u0abd'
    | '\u0ad0'
    | '\u0ae0' .. '\u0ae1'
    | '\u0b05' .. '\u0b0c'
    | '\u0b0f' .. '\u0b10'
    | '\u0b13' .. '\u0b28'
    | '\u0b2a' .. '\u0b30'
    | '\u0b32' .. '\u0b33'
    | '\u0b35' .. '\u0b39'
    | '\u0b3d'
    | '\u0b5c' .. '\u0b5d'
    | '\u0b5f' .. '\u0b61'
    | '\u0b71'
    | '\u0b83'
    | '\u0b85' .. '\u0b8a'
    | '\u0b8e' .. '\u0b90'
    | '\u0b92' .. '\u0b95'
    | '\u0b99' .. '\u0b9a'
    | '\u0b9c'
    | '\u0b9e' .. '\u0b9f'
    | '\u0ba3' .. '\u0ba4'
    | '\u0ba8' .. '\u0baa'
    | '\u0bae' .. '\u0bb5'
    | '\u0bb7' .. '\u0bb9'
    | '\u0c05' .. '\u0c0c'
    | '\u0c0e' .. '\u0c10'
    | '\u0c12' .. '\u0c28'
    | '\u0c2a' .. '\u0c33'
    | '\u0c35' .. '\u0c39'
    | '\u0c60' .. '\u0c61'
    | '\u0c85' .. '\u0c8c'
    | '\u0c8e' .. '\u0c90'
    | '\u0c92' .. '\u0ca8'
    | '\u0caa' .. '\u0cb3'
    | '\u0cb5' .. '\u0cb9'
    | '\u0cbd'
    | '\u0cde'
    | '\u0ce0' .. '\u0ce1'
    | '\u0d05' .. '\u0d0c'
    | '\u0d0e' .. '\u0d10'
    | '\u0d12' .. '\u0d28'
    | '\u0d2a' .. '\u0d39'
    | '\u0d60' .. '\u0d61'
    | '\u0d85' .. '\u0d96'
    | '\u0d9a' .. '\u0db1'
    | '\u0db3' .. '\u0dbb'
    | '\u0dbd'
    | '\u0dc0' .. '\u0dc6'
    | '\u0e01' .. '\u0e30'
    | '\u0e32'
    | '\u0e40' .. '\u0e46'
    | '\u0e81' .. '\u0e82'
    | '\u0e84'
    | '\u0e87' .. '\u0e88'
    | '\u0e8a'
    | '\u0e8d'
    | '\u0e94' .. '\u0e97'
    | '\u0e99' .. '\u0e9f'
    | '\u0ea1' .. '\u0ea3'
    | '\u0ea5'
    | '\u0ea7'
    | '\u0eaa' .. '\u0eab'
    | '\u0ead' .. '\u0eb0'
    | '\u0eb2'
    | '\u0ebd'
    | '\u0ec0' .. '\u0ec4'
    | '\u0ec6'
    | '\u0edc' .. '\u0edd'
    | '\u0f00'
    | '\u0f40' .. '\u0f47'
    | '\u0f49' .. '\u0f6a'
    | '\u0f88' .. '\u0f8b'
    | '\u1000' .. '\u1021'
    | '\u1023' .. '\u1027'
    | '\u1029' .. '\u102a'
    | '\u1050' .. '\u1055'
    | '\u10a0' .. '\u10c5'
    | '\u10d0' .. '\u10f8'
    | '\u1100' .. '\u1159'
    | '\u115f' .. '\u11a2'
    | '\u11a8' .. '\u11f9'
    | '\u1200' .. '\u1206'
    | '\u1208' .. '\u1246'
    | '\u1248'
    | '\u124a' .. '\u124d'
    | '\u1250' .. '\u1256'
    | '\u1258'
    | '\u125a' .. '\u125d'
    | '\u1260' .. '\u1286'
    | '\u1288'
    | '\u128a' .. '\u128d'
    | '\u1290' .. '\u12ae'
    | '\u12b0'
    | '\u12b2' .. '\u12b5'
    | '\u12b8' .. '\u12be'
    | '\u12c0'
    | '\u12c2' .. '\u12c5'
    | '\u12c8' .. '\u12ce'
    | '\u12d0' .. '\u12d6'
    | '\u12d8' .. '\u12ee'
    | '\u12f0' .. '\u130e'
    | '\u1310'
    | '\u1312' .. '\u1315'
    | '\u1318' .. '\u131e'
    | '\u1320' .. '\u1346'
    | '\u1348' .. '\u135a'
    | '\u13a0' .. '\u13f4'
    | '\u1401' .. '\u166c'
    | '\u166f' .. '\u1676'
    | '\u1681' .. '\u169a'
    | '\u16a0' .. '\u16ea'
    | '\u16ee' .. '\u16f0'
    | '\u1700' .. '\u170c'
    | '\u170e' .. '\u1711'
    | '\u1720' .. '\u1731'
    | '\u1740' .. '\u1751'
    | '\u1760' .. '\u176c'
    | '\u176e' .. '\u1770'
    | '\u1780' .. '\u17b3'
    | '\u17d7'
    | '\u17dc'
    | '\u1820' .. '\u1877'
    | '\u1880' .. '\u18a8'
    | '\u1900' .. '\u191c'
    | '\u1950' .. '\u196d'
    | '\u1970' .. '\u1974'
    | '\u1d00' .. '\u1d6b'
    | '\u1e00' .. '\u1e9b'
    | '\u1ea0' .. '\u1ef9'
    | '\u1f00' .. '\u1f15'
    | '\u1f18' .. '\u1f1d'
    | '\u1f20' .. '\u1f45'
    | '\u1f48' .. '\u1f4d'
    | '\u1f50' .. '\u1f57'
    | '\u1f59'
    | '\u1f5b'
    | '\u1f5d'
    | '\u1f5f' .. '\u1f7d'
    | '\u1f80' .. '\u1fb4'
    | '\u1fb6' .. '\u1fbc'
    | '\u1fbe'
    | '\u1fc2' .. '\u1fc4'
    | '\u1fc6' .. '\u1fcc'
    | '\u1fd0' .. '\u1fd3'
    | '\u1fd6' .. '\u1fdb'
    | '\u1fe0' .. '\u1fec'
    | '\u1ff2' .. '\u1ff4'
    | '\u1ff6' .. '\u1ffc'
    | '\u2071'
    | '\u207f'
    | '\u2102'
    | '\u2107'
    | '\u210a' .. '\u2113'
    | '\u2115'
    | '\u2118' .. '\u211d'
    | '\u2124'
    | '\u2126'
    | '\u2128'
    | '\u212a' .. '\u2131'
    | '\u2133' .. '\u2139'
    | '\u213d' .. '\u213f'
    | '\u2145' .. '\u2149'
    | '\u2160' .. '\u2183'
    | '\u3005' .. '\u3007'
    | '\u3021' .. '\u3029'
    | '\u3031' .. '\u3035'
    | '\u3038' .. '\u303c'
    | '\u3041' .. '\u3096'
    | '\u309d' .. '\u309f'
    | '\u30a1' .. '\u30fa'
    | '\u30fc' .. '\u30ff'
    | '\u3105' .. '\u312c'
    | '\u3131' .. '\u318e'
    | '\u31a0' .. '\u31b7'
    | '\u31f0' .. '\u31ff'
    | '\u3400' .. '\u4db5'
    | '\u4e00' .. '\u9fa5'
    | '\ua000' .. '\ua48c'
    | '\uac00' .. '\ud7a3'
    | '\uf900' .. '\ufa2d'
    | '\ufa30' .. '\ufa6a'
    | '\ufb00' .. '\ufb06'
    | '\ufb13' .. '\ufb17'
    | '\ufb1d'
    | '\ufb1f' .. '\ufb28'
    | '\ufb2a' .. '\ufb36'
    | '\ufb38' .. '\ufb3c'
    | '\ufb3e'
    | '\ufb40' .. '\ufb41'
    | '\ufb43' .. '\ufb44'
    | '\ufb46' .. '\ufbb1'
    | '\ufbd3' .. '\ufc5d'
    | '\ufc64' .. '\ufd3d'
    | '\ufd50' .. '\ufd8f'
    | '\ufd92' .. '\ufdc7'
    | '\ufdf0' .. '\ufdf9'
    | '\ufe71'
    | '\ufe73'
    | '\ufe77'
    | '\ufe79'
    | '\ufe7b'
    | '\ufe7d'
    | '\ufe7f' .. '\ufefc'
    | '\uff21' .. '\uff3a'
    | '\uff41' .. '\uff5a'
    | '\uff66' .. '\uff9d'
    | '\uffa0' .. '\uffbe'
    | '\uffc2' .. '\uffc7'
    | '\uffca' .. '\uffcf'
    | '\uffd2' .. '\uffd7'
    | '\uffda' .. '\uffdc'
    | '\ud800' '\udc00' .. '\udc0a'
    | '\ud800' '\udc0d' .. '\udc25'
    | '\ud800' '\udc28' .. '\udc39'
    | '\ud800' '\udc3c' .. '\udc3c'
    | '\ud800' '\udc3f' .. '\udc4c'
    | '\ud800' '\udc50' .. '\udc5c'
    | '\ud800' '\udc80' .. '\udcf9'
    | '\ud800' '\udf00' .. '\udf1d'
    | '\ud800' '\udf30' .. '\udf49'
    | '\ud800' '\udf80' .. '\udf9c'
    | '\ud801' '\ue000' .. '\ue09c'
    | '\ud802' '\ue400' .. '\ue404'
    | '\ud802' '\u0808'
    | '\ud802' '\ue40a' .. '\ue434'
    | '\ud802' '\ue437' .. '\ue437'
    | '\ud802' '\u083c'
    | '\ud802' '\u083f'
    | '\ud835' '\ub000' .. '\ub053'
    | '\ud835' '\ub056' .. '\ub09b'
    | '\ud835' '\ub09e' .. '\ub09e'
    | '\ud835' '\ud4a2'
    | '\ud835' '\ub0a5' .. '\ub0a5'
    | '\ud835' '\ub0a9' .. '\ub0ab'
    | '\ud835' '\ub0ae' .. '\ub0b8'
    | '\ud835' '\ud4bb'
    | '\ud835' '\ub0bd' .. '\ub0c2'
    | '\ud835' '\ub0c5' .. '\ub104'
    | '\ud835' '\ub107' .. '\ub109'
    | '\ud835' '\ub10d' .. '\ub113'
    | '\ud835' '\ub116' .. '\ub11b'
    | '\ud835' '\ub11e' .. '\ub138'
    | '\ud835' '\ub13b' .. '\ub13d'
    | '\ud835' '\ub140' .. '\ub143'
    | '\ud835' '\ud546'
    | '\ud835' '\ub14a' .. '\ub14f'
    | '\ud835' '\ub152' .. '\ub2a2'
    | '\ud835' '\ub2a8' .. '\ub2bf'
    | '\ud835' '\ub2c2' .. '\ub2d9'
    | '\ud835' '\ub2dc' .. '\ub2f9'
    | '\ud835' '\ub2fc' .. '\ub313'
    | '\ud835' '\ub316' .. '\ub333'
    | '\ud835' '\ub336' .. '\ub34d'
    | '\ud835' '\ub350' .. '\ub36d'
    | '\ud835' '\ub370' .. '\ub387'
    | '\ud835' '\ub38a' .. '\ub3a7'
    | '\ud835' '\ub3aa' .. '\ub3c1'
    | '\ud835' '\ub3c4' .. '\ub3c8'
    | '\ud840' '\udc00' .. '\udffe'
    | '\ud841' '\ue000' .. '\ue3fe'
    | '\ud842' '\ue400' .. '\ue7fe'
    | '\ud843' '\ue800' .. '\uebfe'
    | '\ud844' '\uec00' .. '\ueffe'
    | '\ud845' '\uf000' .. '\uf3fe'
    | '\ud846' '\uf400' .. '\uf7fe'
    | '\ud847' '\uf800' .. '\ufbfe'
    | '\ud848' '\ufc00' .. '\ufffe'
    | '\ud849' '\u0000' .. '\u03fe'
    | '\ud84a' '\u0400' .. '\u07fe'
    | '\ud84b' '\u0800' .. '\u0bfe'
    | '\ud84c' '\u0c00' .. '\u0ffe'
    | '\ud84d' '\u1000' .. '\u13fe'
    | '\ud84e' '\u1400' .. '\u17fe'
    | '\ud84f' '\u1800' .. '\u1bfe'
    | '\ud850' '\u1c00' .. '\u1ffe'
    | '\ud851' '\u2000' .. '\u23fe'
    | '\ud852' '\u2400' .. '\u27fe'
    | '\ud853' '\u2800' .. '\u2bfe'
    | '\ud854' '\u2c00' .. '\u2ffe'
    | '\ud855' '\u3000' .. '\u33fe'
    | '\ud856' '\u3400' .. '\u37fe'
    | '\ud857' '\u3800' .. '\u3bfe'
    | '\ud858' '\u3c00' .. '\u3ffe'
    | '\ud859' '\u4000' .. '\u43fe'
    | '\ud85a' '\u4400' .. '\u47fe'
    | '\ud85b' '\u4800' .. '\u4bfe'
    | '\ud85c' '\u4c00' .. '\u4ffe'
    | '\ud85d' '\u5000' .. '\u53fe'
    | '\ud85e' '\u5400' .. '\u57fe'
    | '\ud85f' '\u5800' .. '\u5bfe'
    | '\ud860' '\u5c00' .. '\u5ffe'
    | '\ud861' '\u6000' .. '\u63fe'
    | '\ud862' '\u6400' .. '\u67fe'
    | '\ud863' '\u6800' .. '\u6bfe'
    | '\ud864' '\u6c00' .. '\u6ffe'
    | '\ud865' '\u7000' .. '\u73fe'
    | '\ud866' '\u7400' .. '\u77fe'
    | '\ud867' '\u7800' .. '\u7bfe'
    | '\ud868' '\u7c00' .. '\u7ffe'
    | '\ud869' '\u8000' .. '\u82d5'
    | '\ud87e' '\ud400' .. '\ud61c'
    ;

fragment XID_Continue:
      '\u0030' .. '\u0039'
    | '\u0041' .. '\u005a'
    | '\u005f'
    | '\u0061' .. '\u007a'
    | '\u00aa'
    | '\u00b5'
    | '\u00b7'
    | '\u00ba'
    | '\u00c0' .. '\u00d6'
    | '\u00d8' .. '\u00f6'
    | '\u00f8' .. '\u0236'
    | '\u0250' .. '\u02c1'
    | '\u02c6' .. '\u02d1'
    | '\u02e0' .. '\u02e4'
    | '\u02ee'
    | '\u0300' .. '\u0357'
    | '\u035d' .. '\u036f'
    | '\u0386'
    | '\u0388' .. '\u038a'
    | '\u038c'
    | '\u038e' .. '\u03a1'
    | '\u03a3' .. '\u03ce'
    | '\u03d0' .. '\u03f5'
    | '\u03f7' .. '\u03fb'
    | '\u0400' .. '\u0481'
    | '\u0483' .. '\u0486'
    | '\u048a' .. '\u04ce'
    | '\u04d0' .. '\u04f5'
    | '\u04f8' .. '\u04f9'
    | '\u0500' .. '\u050f'
    | '\u0531' .. '\u0556'
    | '\u0559'
    | '\u0561' .. '\u0587'
    | '\u0591' .. '\u05a1'
    | '\u05a3' .. '\u05b9'
    | '\u05bb' .. '\u05bd'
    | '\u05bf'
    | '\u05c1' .. '\u05c2'
    | '\u05c4'
    | '\u05d0' .. '\u05ea'
    | '\u05f0' .. '\u05f2'
    | '\u0610' .. '\u0615'
    | '\u0621' .. '\u063a'
    | '\u0640' .. '\u0658'
    | '\u0660' .. '\u0669'
    | '\u066e' .. '\u06d3'
    | '\u06d5' .. '\u06dc'
    | '\u06df' .. '\u06e8'
    | '\u06ea' .. '\u06fc'
    | '\u06ff'
    | '\u0710' .. '\u074a'
    | '\u074d' .. '\u074f'
    | '\u0780' .. '\u07b1'
    | '\u0901' .. '\u0939'
    | '\u093c' .. '\u094d'
    | '\u0950' .. '\u0954'
    | '\u0958' .. '\u0963'
    | '\u0966' .. '\u096f'
    | '\u0981' .. '\u0983'
    | '\u0985' .. '\u098c'
    | '\u098f' .. '\u0990'
    | '\u0993' .. '\u09a8'
    | '\u09aa' .. '\u09b0'
    | '\u09b2'
    | '\u09b6' .. '\u09b9'
    | '\u09bc' .. '\u09c4'
    | '\u09c7' .. '\u09c8'
    | '\u09cb' .. '\u09cd'
    | '\u09d7'
    | '\u09dc' .. '\u09dd'
    | '\u09df' .. '\u09e3'
    | '\u09e6' .. '\u09f1'
    | '\u0a01' .. '\u0a03'
    | '\u0a05' .. '\u0a0a'
    | '\u0a0f' .. '\u0a10'
    | '\u0a13' .. '\u0a28'
    | '\u0a2a' .. '\u0a30'
    | '\u0a32' .. '\u0a33'
    | '\u0a35' .. '\u0a36'
    | '\u0a38' .. '\u0a39'
    | '\u0a3c'
    | '\u0a3e' .. '\u0a42'
    | '\u0a47' .. '\u0a48'
    | '\u0a4b' .. '\u0a4d'
    | '\u0a59' .. '\u0a5c'
    | '\u0a5e'
    | '\u0a66' .. '\u0a74'
    | '\u0a81' .. '\u0a83'
    | '\u0a85' .. '\u0a8d'
    | '\u0a8f' .. '\u0a91'
    | '\u0a93' .. '\u0aa8'
    | '\u0aaa' .. '\u0ab0'
    | '\u0ab2' .. '\u0ab3'
    | '\u0ab5' .. '\u0ab9'
    | '\u0abc' .. '\u0ac5'
    | '\u0ac7' .. '\u0ac9'
    | '\u0acb' .. '\u0acd'
    | '\u0ad0'
    | '\u0ae0' .. '\u0ae3'
    | '\u0ae6' .. '\u0aef'
    | '\u0b01' .. '\u0b03'
    | '\u0b05' .. '\u0b0c'
    | '\u0b0f' .. '\u0b10'
    | '\u0b13' .. '\u0b28'
    | '\u0b2a' .. '\u0b30'
    | '\u0b32' .. '\u0b33'
    | '\u0b35' .. '\u0b39'
    | '\u0b3c' .. '\u0b43'
    | '\u0b47' .. '\u0b48'
    | '\u0b4b' .. '\u0b4d'
    | '\u0b56' .. '\u0b57'
    | '\u0b5c' .. '\u0b5d'
    | '\u0b5f' .. '\u0b61'
    | '\u0b66' .. '\u0b6f'
    | '\u0b71'
    | '\u0b82' .. '\u0b83'
    | '\u0b85' .. '\u0b8a'
    | '\u0b8e' .. '\u0b90'
    | '\u0b92' .. '\u0b95'
    | '\u0b99' .. '\u0b9a'
    | '\u0b9c'
    | '\u0b9e' .. '\u0b9f'
    | '\u0ba3' .. '\u0ba4'
    | '\u0ba8' .. '\u0baa'
    | '\u0bae' .. '\u0bb5'
    | '\u0bb7' .. '\u0bb9'
    | '\u0bbe' .. '\u0bc2'
    | '\u0bc6' .. '\u0bc8'
    | '\u0bca' .. '\u0bcd'
    | '\u0bd7'
    | '\u0be7' .. '\u0bef'
    | '\u0c01' .. '\u0c03'
    | '\u0c05' .. '\u0c0c'
    | '\u0c0e' .. '\u0c10'
    | '\u0c12' .. '\u0c28'
    | '\u0c2a' .. '\u0c33'
    | '\u0c35' .. '\u0c39'
    | '\u0c3e' .. '\u0c44'
    | '\u0c46' .. '\u0c48'
    | '\u0c4a' .. '\u0c4d'
    | '\u0c55' .. '\u0c56'
    | '\u0c60' .. '\u0c61'
    | '\u0c66' .. '\u0c6f'
    | '\u0c82' .. '\u0c83'
    | '\u0c85' .. '\u0c8c'
    | '\u0c8e' .. '\u0c90'
    | '\u0c92' .. '\u0ca8'
    | '\u0caa' .. '\u0cb3'
    | '\u0cb5' .. '\u0cb9'
    | '\u0cbc' .. '\u0cc4'
    | '\u0cc6' .. '\u0cc8'
    | '\u0cca' .. '\u0ccd'
    | '\u0cd5' .. '\u0cd6'
    | '\u0cde'
    | '\u0ce0' .. '\u0ce1'
    | '\u0ce6' .. '\u0cef'
    | '\u0d02' .. '\u0d03'
    | '\u0d05' .. '\u0d0c'
    | '\u0d0e' .. '\u0d10'
    | '\u0d12' .. '\u0d28'
    | '\u0d2a' .. '\u0d39'
    | '\u0d3e' .. '\u0d43'
    | '\u0d46' .. '\u0d48'
    | '\u0d4a' .. '\u0d4d'
    | '\u0d57'
    | '\u0d60' .. '\u0d61'
    | '\u0d66' .. '\u0d6f'
    | '\u0d82' .. '\u0d83'
    | '\u0d85' .. '\u0d96'
    | '\u0d9a' .. '\u0db1'
    | '\u0db3' .. '\u0dbb'
    | '\u0dbd'
    | '\u0dc0' .. '\u0dc6'
    | '\u0dca'
    | '\u0dcf' .. '\u0dd4'
    | '\u0dd6'
    | '\u0dd8' .. '\u0ddf'
    | '\u0df2' .. '\u0df3'
    | '\u0e01' .. '\u0e3a'
    | '\u0e40' .. '\u0e4e'
    | '\u0e50' .. '\u0e59'
    | '\u0e81' .. '\u0e82'
    | '\u0e84'
    | '\u0e87' .. '\u0e88'
    | '\u0e8a'
    | '\u0e8d'
    | '\u0e94' .. '\u0e97'
    | '\u0e99' .. '\u0e9f'
    | '\u0ea1' .. '\u0ea3'
    | '\u0ea5'
    | '\u0ea7'
    | '\u0eaa' .. '\u0eab'
    | '\u0ead' .. '\u0eb9'
    | '\u0ebb' .. '\u0ebd'
    | '\u0ec0' .. '\u0ec4'
    | '\u0ec6'
    | '\u0ec8' .. '\u0ecd'
    | '\u0ed0' .. '\u0ed9'
    | '\u0edc' .. '\u0edd'
    | '\u0f00'
    | '\u0f18' .. '\u0f19'
    | '\u0f20' .. '\u0f29'
    | '\u0f35'
    | '\u0f37'
    | '\u0f39'
    | '\u0f3e' .. '\u0f47'
    | '\u0f49' .. '\u0f6a'
    | '\u0f71' .. '\u0f84'
    | '\u0f86' .. '\u0f8b'
    | '\u0f90' .. '\u0f97'
    | '\u0f99' .. '\u0fbc'
    | '\u0fc6'
    | '\u1000' .. '\u1021'
    | '\u1023' .. '\u1027'
    | '\u1029' .. '\u102a'
    | '\u102c' .. '\u1032'
    | '\u1036' .. '\u1039'
    | '\u1040' .. '\u1049'
    | '\u1050' .. '\u1059'
    | '\u10a0' .. '\u10c5'
    | '\u10d0' .. '\u10f8'
    | '\u1100' .. '\u1159'
    | '\u115f' .. '\u11a2'
    | '\u11a8' .. '\u11f9'
    | '\u1200' .. '\u1206'
    | '\u1208' .. '\u1246'
    | '\u1248'
    | '\u124a' .. '\u124d'
    | '\u1250' .. '\u1256'
    | '\u1258'
    | '\u125a' .. '\u125d'
    | '\u1260' .. '\u1286'
    | '\u1288'
    | '\u128a' .. '\u128d'
    | '\u1290' .. '\u12ae'
    | '\u12b0'
    | '\u12b2' .. '\u12b5'
    | '\u12b8' .. '\u12be'
    | '\u12c0'
    | '\u12c2' .. '\u12c5'
    | '\u12c8' .. '\u12ce'
    | '\u12d0' .. '\u12d6'
    | '\u12d8' .. '\u12ee'
    | '\u12f0' .. '\u130e'
    | '\u1310'
    | '\u1312' .. '\u1315'
    | '\u1318' .. '\u131e'
    | '\u1320' .. '\u1346'
    | '\u1348' .. '\u135a'
    | '\u1369' .. '\u1371'
    | '\u13a0' .. '\u13f4'
    | '\u1401' .. '\u166c'
    | '\u166f' .. '\u1676'
    | '\u1681' .. '\u169a'
    | '\u16a0' .. '\u16ea'
    | '\u16ee' .. '\u16f0'
    | '\u1700' .. '\u170c'
    | '\u170e' .. '\u1714'
    | '\u1720' .. '\u1734'
    | '\u1740' .. '\u1753'
    | '\u1760' .. '\u176c'
    | '\u176e' .. '\u1770'
    | '\u1772' .. '\u1773'
    | '\u1780' .. '\u17b3'
    | '\u17b6' .. '\u17d3'
    | '\u17d7'
    | '\u17dc' .. '\u17dd'
    | '\u17e0' .. '\u17e9'
    | '\u180b' .. '\u180d'
    | '\u1810' .. '\u1819'
    | '\u1820' .. '\u1877'
    | '\u1880' .. '\u18a9'
    | '\u1900' .. '\u191c'
    | '\u1920' .. '\u192b'
    | '\u1930' .. '\u193b'
    | '\u1946' .. '\u196d'
    | '\u1970' .. '\u1974'
    | '\u1d00' .. '\u1d6b'
    | '\u1e00' .. '\u1e9b'
    | '\u1ea0' .. '\u1ef9'
    | '\u1f00' .. '\u1f15'
    | '\u1f18' .. '\u1f1d'
    | '\u1f20' .. '\u1f45'
    | '\u1f48' .. '\u1f4d'
    | '\u1f50' .. '\u1f57'
    | '\u1f59'
    | '\u1f5b'
    | '\u1f5d'
    | '\u1f5f' .. '\u1f7d'
    | '\u1f80' .. '\u1fb4'
    | '\u1fb6' .. '\u1fbc'
    | '\u1fbe'
    | '\u1fc2' .. '\u1fc4'
    | '\u1fc6' .. '\u1fcc'
    | '\u1fd0' .. '\u1fd3'
    | '\u1fd6' .. '\u1fdb'
    | '\u1fe0' .. '\u1fec'
    | '\u1ff2' .. '\u1ff4'
    | '\u1ff6' .. '\u1ffc'
    | '\u203f' .. '\u2040'
    | '\u2054'
    | '\u2071'
    | '\u207f'
    | '\u20d0' .. '\u20dc'
    | '\u20e1'
    | '\u20e5' .. '\u20ea'
    | '\u2102'
    | '\u2107'
    | '\u210a' .. '\u2113'
    | '\u2115'
    | '\u2118' .. '\u211d'
    | '\u2124'
    | '\u2126'
    | '\u2128'
    | '\u212a' .. '\u2131'
    | '\u2133' .. '\u2139'
    | '\u213d' .. '\u213f'
    | '\u2145' .. '\u2149'
    | '\u2160' .. '\u2183'
    | '\u3005' .. '\u3007'
    | '\u3021' .. '\u302f'
    | '\u3031' .. '\u3035'
    | '\u3038' .. '\u303c'
    | '\u3041' .. '\u3096'
    | '\u3099' .. '\u309a'
    | '\u309d' .. '\u309f'
    | '\u30a1' .. '\u30ff'
    | '\u3105' .. '\u312c'
    | '\u3131' .. '\u318e'
    | '\u31a0' .. '\u31b7'
    | '\u31f0' .. '\u31ff'
    | '\u3400' .. '\u4db5'
    | '\u4e00' .. '\u9fa5'
    | '\ua000' .. '\ua48c'
    | '\uac00' .. '\ud7a3'
    | '\uf900' .. '\ufa2d'
    | '\ufa30' .. '\ufa6a'
    | '\ufb00' .. '\ufb06'
    | '\ufb13' .. '\ufb17'
    | '\ufb1d' .. '\ufb28'
    | '\ufb2a' .. '\ufb36'
    | '\ufb38' .. '\ufb3c'
    | '\ufb3e'
    | '\ufb40' .. '\ufb41'
    | '\ufb43' .. '\ufb44'
    | '\ufb46' .. '\ufbb1'
    | '\ufbd3' .. '\ufc5d'
    | '\ufc64' .. '\ufd3d'
    | '\ufd50' .. '\ufd8f'
    | '\ufd92' .. '\ufdc7'
    | '\ufdf0' .. '\ufdf9'
    | '\ufe00' .. '\ufe0f'
    | '\ufe20' .. '\ufe23'
    | '\ufe33' .. '\ufe34'
    | '\ufe4d' .. '\ufe4f'
    | '\ufe71'
    | '\ufe73'
    | '\ufe77'
    | '\ufe79'
    | '\ufe7b'
    | '\ufe7d'
    | '\ufe7f' .. '\ufefc'
    | '\uff10' .. '\uff19'
    | '\uff21' .. '\uff3a'
    | '\uff3f'
    | '\uff41' .. '\uff5a'
    | '\uff65' .. '\uffbe'
    | '\uffc2' .. '\uffc7'
    | '\uffca' .. '\uffcf'
    | '\uffd2' .. '\uffd7'
    | '\uffda' .. '\uffdc'
    | '\ud800' '\udc00' .. '\udc0a'
    | '\ud800' '\udc0d' .. '\udc25'
    | '\ud800' '\udc28' .. '\udc39'
    | '\ud800' '\udc3c' .. '\udc3c'
    | '\ud800' '\udc3f' .. '\udc4c'
    | '\ud800' '\udc50' .. '\udc5c'
    | '\ud800' '\udc80' .. '\udcf9'
    | '\ud800' '\udf00' .. '\udf1d'
    | '\ud800' '\udf30' .. '\udf49'
    | '\ud800' '\udf80' .. '\udf9c'
    | '\ud801' '\ue000' .. '\ue09c'
    | '\ud801' '\ue0a0' .. '\ue0a8'
    | '\ud802' '\ue400' .. '\ue404'
    | '\ud802' '\u0808'
    | '\ud802' '\ue40a' .. '\ue434'
    | '\ud802' '\ue437' .. '\ue437'
    | '\ud802' '\u083c'
    | '\ud802' '\u083f'
    | '\ud834' '\uad65' .. '\uad68'
    | '\ud834' '\uad6d' .. '\uad71'
    | '\ud834' '\uad7b' .. '\uad81'
    | '\ud834' '\uad85' .. '\uad8a'
    | '\ud834' '\uadaa' .. '\uadac'
    | '\ud835' '\ub000' .. '\ub053'
    | '\ud835' '\ub056' .. '\ub09b'
    | '\ud835' '\ub09e' .. '\ub09e'
    | '\ud835' '\ud4a2'
    | '\ud835' '\ub0a5' .. '\ub0a5'
    | '\ud835' '\ub0a9' .. '\ub0ab'
    | '\ud835' '\ub0ae' .. '\ub0b8'
    | '\ud835' '\ud4bb'
    | '\ud835' '\ub0bd' .. '\ub0c2'
    | '\ud835' '\ub0c5' .. '\ub104'
    | '\ud835' '\ub107' .. '\ub109'
    | '\ud835' '\ub10d' .. '\ub113'
    | '\ud835' '\ub116' .. '\ub11b'
    | '\ud835' '\ub11e' .. '\ub138'
    | '\ud835' '\ub13b' .. '\ub13d'
    | '\ud835' '\ub140' .. '\ub143'
    | '\ud835' '\ud546'
    | '\ud835' '\ub14a' .. '\ub14f'
    | '\ud835' '\ub152' .. '\ub2a2'
    | '\ud835' '\ub2a8' .. '\ub2bf'
    | '\ud835' '\ub2c2' .. '\ub2d9'
    | '\ud835' '\ub2dc' .. '\ub2f9'
    | '\ud835' '\ub2fc' .. '\ub313'
    | '\ud835' '\ub316' .. '\ub333'
    | '\ud835' '\ub336' .. '\ub34d'
    | '\ud835' '\ub350' .. '\ub36d'
    | '\ud835' '\ub370' .. '\ub387'
    | '\ud835' '\ub38a' .. '\ub3a7'
    | '\ud835' '\ub3aa' .. '\ub3c1'
    | '\ud835' '\ub3c4' .. '\ub3c8'
    | '\ud835' '\ub3ce' .. '\ub3fe'
    | '\ud840' '\udc00' .. '\udffe'
    | '\ud841' '\ue000' .. '\ue3fe'
    | '\ud842' '\ue400' .. '\ue7fe'
    | '\ud843' '\ue800' .. '\uebfe'
    | '\ud844' '\uec00' .. '\ueffe'
    | '\ud845' '\uf000' .. '\uf3fe'
    | '\ud846' '\uf400' .. '\uf7fe'
    | '\ud847' '\uf800' .. '\ufbfe'
    | '\ud848' '\ufc00' .. '\ufffe'
    | '\ud849' '\u0000' .. '\u03fe'
    | '\ud84a' '\u0400' .. '\u07fe'
    | '\ud84b' '\u0800' .. '\u0bfe'
    | '\ud84c' '\u0c00' .. '\u0ffe'
    | '\ud84d' '\u1000' .. '\u13fe'
    | '\ud84e' '\u1400' .. '\u17fe'
    | '\ud84f' '\u1800' .. '\u1bfe'
    | '\ud850' '\u1c00' .. '\u1ffe'
    | '\ud851' '\u2000' .. '\u23fe'
    | '\ud852' '\u2400' .. '\u27fe'
    | '\ud853' '\u2800' .. '\u2bfe'
    | '\ud854' '\u2c00' .. '\u2ffe'
    | '\ud855' '\u3000' .. '\u33fe'
    | '\ud856' '\u3400' .. '\u37fe'
    | '\ud857' '\u3800' .. '\u3bfe'
    | '\ud858' '\u3c00' .. '\u3ffe'
    | '\ud859' '\u4000' .. '\u43fe'
    | '\ud85a' '\u4400' .. '\u47fe'
    | '\ud85b' '\u4800' .. '\u4bfe'
    | '\ud85c' '\u4c00' .. '\u4ffe'
    | '\ud85d' '\u5000' .. '\u53fe'
    | '\ud85e' '\u5400' .. '\u57fe'
    | '\ud85f' '\u5800' .. '\u5bfe'
    | '\ud860' '\u5c00' .. '\u5ffe'
    | '\ud861' '\u6000' .. '\u63fe'
    | '\ud862' '\u6400' .. '\u67fe'
    | '\ud863' '\u6800' .. '\u6bfe'
    | '\ud864' '\u6c00' .. '\u6ffe'
    | '\ud865' '\u7000' .. '\u73fe'
    | '\ud866' '\u7400' .. '\u77fe'
    | '\ud867' '\u7800' .. '\u7bfe'
    | '\ud868' '\u7c00' .. '\u7ffe'
    | '\ud869' '\u8000' .. '\u82d5'
    | '\ud87e' '\ud400' .. '\ud61c'
    | '\udb40' '\udd00' .. '\uddee'
    ;