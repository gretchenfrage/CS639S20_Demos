#include "Laplacian.h"
#include "Timer.h"

#include <string>

void ComputeLaplacian(const float (&u)[XDIM][YDIM][ZDIM], float (&Lu)[XDIM][YDIM][ZDIM], int line)
{   
    Timer timer;
    timer.Start();
    
#pragma omp parallel for
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
        Lu[i][j][k] =
            -6 * u[i][j][k]
            + u[i+1][j][k]
            + u[i-1][j][k]
            + u[i][j+1][k]
            + u[i][j-1][k]
            + u[i][j][k+1]
            + u[i][j][k-1];
            
    timer.Stop("KERNEL ComputeLaplacian() on line  " + std::to_string(line) + " : Time = ");
}
