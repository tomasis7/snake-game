#include "sprites.hpp"

#include <fstream>
#include <sstream>

bool parseSprites(const std::string& text, std::vector<Sprite>& out, std::string& err) {
    std::istringstream in(text);
    std::string line;
    Sprite cur;
    enum { None, Palette, Rows } state = None;
    while (std::getline(in, line)) {
        if (!line.empty() && line.back() == '\r') line.pop_back();
        if (state == None) {
            if (line.empty()) continue;
            std::istringstream ls(line);
            std::string kw;
            cur = Sprite{};
            ls >> kw >> cur.name >> cur.cols >> cur.rowsCount;
            if (kw != "sprite" || cur.name.empty() || cur.cols <= 0 || cur.rowsCount <= 0) {
                err = "bad sprite header: " + line;
                return false;
            }
            state = Palette;
        } else if (state == Palette) {
            if (line == "rows") {
                state = Rows;
            } else if (line.size() >= 9 && line[1] == ' ') {
                cur.palette[line[0]] = line.substr(2);
            } else {
                err = "bad palette line in " + cur.name + ": " + line;
                return false;
            }
        } else {
            if (line == "end") {
                if ((int)cur.rows.size() != cur.rowsCount) {
                    err = "row count mismatch in " + cur.name;
                    return false;
                }
                out.push_back(cur);
                state = None;
            } else {
                if ((int)line.size() != cur.cols) {
                    err = "row width mismatch in " + cur.name;
                    return false;
                }
                cur.rows.push_back(line);
            }
        }
    }
    if (state != None) {
        err = "unterminated sprite";
        return false;
    }
    return true;
}

bool loadSprites(const std::string& path, std::vector<Sprite>& out, std::string& err) {
    std::ifstream f(path, std::ios::binary);
    if (!f) {
        err = "cannot open " + path;
        return false;
    }
    std::stringstream ss;
    ss << f.rdbuf();
    return parseSprites(ss.str(), out, err);
}

const Sprite* findSprite(const std::vector<Sprite>& all, const std::string& name) {
    for (auto& s : all)
        if (s.name == name) return &s;
    return nullptr;
}
