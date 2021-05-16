#pragma once
#include "rust/cxx.h"
#include <oead/types.h>

class U8 : public oead::U8 {
    public:
        uint8_t v() const;
};

class U16 : public oead::U16 {
    public:
        uint16_t v() const;
};

class U32: public oead::U32 {
    public:
        uint32_t v() const;
};

class U64: public oead::U64 {
    public:
        uint64_t v() const;
};

class S8 : public oead::S8 {
    public:
        int8_t v() const;
};

class S16 : public oead::S16 {
    public:
        int16_t v() const;
};

class S32: public oead::S32 {
    public:
        int32_t v() const;
};

class S64: public oead::S64 {
    public:
        int64_t v() const;
};

class F32: public oead::F32 {
    public:
        float v() const;
};

class F64: public oead::F64 {
    public:
        double v() const;
};
