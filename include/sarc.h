#pragma once
#include "rust/cxx.h"
#include <oead/sarc.h>
#include <oead/yaz0.h>
#include <memory>

class Sarc
{
public:
    explicit Sarc(const rust::Slice<const uint8_t> data);
    uint32_t get_offset() const;
    size_t guess_align() const;
    uint16_t num_files() const;
    bool big_endian() const;
    bool files_eq(const Sarc &other) const;
    rust::Slice<const uint8_t> get_file_data(const rust::Str name) const;
    rust::Slice<const uint8_t> idx_file_data(const uint16_t idx) const;
    rust::Str idx_file_name(const uint16_t idx) const;
    oead::Sarc inner;
};

std::unique_ptr<Sarc> sarc_from_binary(rust::Slice<const uint8_t> data);

struct SarcWriteResult;
class SarcWriter : public oead::SarcWriter
{
public:
    using oead::SarcWriter::SarcWriter;
    void SetEndianness(bool big_endian);
    void SetMode(bool legacy);
    void SetFile(rust::Str name, rust::Vec<uint8_t> data);
    bool DelFile(rust::Str name);
    bool FilesEqual(const SarcWriter &other) const;
    size_t NumFiles() const;
    SarcWriteResult Write();
};

std::unique_ptr<SarcWriter> WriterFromSarc(const Sarc& archive);
std::unique_ptr<SarcWriter> NewSarcWriter(bool big_endian, bool legacy);
