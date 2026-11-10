#ifndef ONIGURUMA_H
#define ONIGURUMA_H 1

#include "ruby/encoding.h"

#ifdef __cplusplus
extern "C" {
#endif

#define ONIGMO_VERSION_MAJOR 6
#define ONIGMO_VERSION_MINOR 1
#define ONIGMO_VERSION_TEENY 3
#define ONIGURUMA
#define ONIGURUMA_VERSION_MAJOR ONIGMO_VERSION_MAJOR
#define ONIGURUMA_VERSION_MINOR ONIGMO_VERSION_MINOR
#define ONIGURUMA_VERSION_TEENY ONIGMO_VERSION_TEENY

typedef unsigned char OnigUChar;
#ifndef UChar
#define UChar OnigUChar
#endif
typedef rb_encoding *OnigEncoding;

#define ONIGERR_TYPE_BUG -6
#define ONIGERR_INVALID_CHAR_PROPERTY_NAME -223

/* Each property's number and code point ranges come from the Unicode tables
 * metorex's own patterns match against, which are Onigmo's. */
int rb_metorex_onigenc_property_name_to_ctype(rb_encoding *encoding, const OnigUChar *start, const OnigUChar *end);
int rb_metorex_onigenc_get_ctype_code_range(rb_encoding *encoding, unsigned int ctype, OnigCodePoint *single_byte_out, const OnigCodePoint **ranges);
#define ONIGENC_PROPERTY_NAME_TO_CTYPE(enc, p, end) rb_metorex_onigenc_property_name_to_ctype((enc), (p), (end))
#define ONIGENC_GET_CTYPE_CODE_RANGE(enc, ctype, sbout, ranges) \
  rb_metorex_onigenc_get_ctype_code_range((enc), (ctype), (sbout), (ranges))

#ifdef __cplusplus
}
#endif

#endif /* ONIGURUMA_H */
