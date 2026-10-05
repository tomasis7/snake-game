#pragma once
#include <vector>

struct Camera {
    double scale = 1, centerX = 0, centerY = 0;
};
struct WorldRect {
    double left = 0, right = 0, top = 0, bottom = 0;
};

Camera fitCamera(const std::vector<double>& xs, const std::vector<double>& ys, double viewportW,
                 double viewportH, double padding, double minScale, double maxScale);
double advanceKillLine(double prev, double leaderX, double maxGap);
WorldRect visibleWorldRect(const Camera& cam, double viewportW, double viewportH, double margin);
bool intersectsRect(double x, double y, double w, double h, const WorldRect& r);
