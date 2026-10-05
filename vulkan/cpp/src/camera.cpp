#include "camera.hpp"

#include <algorithm>

Camera fitCamera(const std::vector<double>& xs, const std::vector<double>& ys, double viewportW,
                 double viewportH, double padding, double minScale, double maxScale) {
    double minX = *std::min_element(xs.begin(), xs.end()) - padding;
    double maxX = *std::max_element(xs.begin(), xs.end()) + padding;
    double minY = *std::min_element(ys.begin(), ys.end()) - padding;
    double maxY = *std::max_element(ys.begin(), ys.end()) + padding;
    double boxW = std::max(1.0, maxX - minX);
    double boxH = std::max(1.0, maxY - minY);
    double scale = std::max(minScale, std::min(maxScale, std::min(viewportW / boxW, viewportH / boxH)));
    return {scale, (minX + maxX) / 2, (minY + maxY) / 2};
}

double advanceKillLine(double prev, double leaderX, double maxGap) {
    return std::max(prev, leaderX - maxGap);
}

WorldRect visibleWorldRect(const Camera& cam, double viewportW, double viewportH, double margin) {
    double halfW = viewportW / 2 / cam.scale + margin;
    double halfH = viewportH / 2 / cam.scale + margin;
    return {cam.centerX - halfW, cam.centerX + halfW, cam.centerY - halfH, cam.centerY + halfH};
}

bool intersectsRect(double x, double y, double w, double h, const WorldRect& r) {
    return x + w >= r.left && x <= r.right && y + h >= r.top && y <= r.bottom;
}
