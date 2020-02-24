#include "Laplacian.h"
#include "Parameters.h"
#include "PointwiseOps.h"
#include "Reductions.h"
#include "Utilities.h"

#include <iostream>
#include <utility>
#include <algorithm>

using array_t = float (&) [XDIM][YDIM][ZDIM];
using std::min;
using std::max;
using std::abs;
using std::cout;
using std::to_string;
using std::endl;

array_t new_buffer() {
    float *raw = new float [XDIM*YDIM*ZDIM];
    array_t arr = reinterpret_cast<array_t>(*raw);
    
    return arr;
}

struct DoubleBuffer
{
    // buffer we're reading from
    array_t a = new_buffer();
    
    // buffer we're writing to
    array_t b = new_buffer();
    
    void swap() {
        std::swap(a, b);
    }
};

void ConjugateGradients(
    const float (&x)[XDIM][YDIM][ZDIM],
    const float (&f)[XDIM][YDIM][ZDIM],
    const bool writeIterations)
{
    DoubleBuffer bufs;
    
    int D = 0;
    
    float nu = 0.;
    double rho = 0.;
    double sigma = 0.;
            
#pragma omp parallel for reduction(max:nu) reduction(+:rho) reduction(+:sigma)
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
    {
        float p[3][3][3];
        
        int i2j2k2[7][3] = {
            {  0,  0,  0 },
            {  1,  0,  0 },
            { -1,  0,  0 },
            {  0,  1,  0 },
            {  0, -1,  0 },
            {  0,  0,  1 },
            {  0,  0, -1 },
        };
        for (int n = 0; n < 7; n++) {
            int i2 = i2j2k2[n][0];
            int j2 = i2j2k2[n][1];
            int k2 = i2j2k2[n][2];
            
            int i3 = i2 + i;
            int j3 = j2 + j;
            int k3 = k2 + k;
            
            p[i2+1][j2+1][k2+1] = 
                -6 * x[i3][j3][k3]
                + x[min(i3+1, XDIM-1)][j3               ][k3               ]
                + x[max(i3-1, 0)     ][j3               ][k3               ]
                + x[i3               ][min(j3+1, YDIM-1)][k3               ]
                + x[i3               ][max(j3-1, 0     )][k3               ]
                + x[i3               ][j3               ][min(k3+1, ZDIM-1)]
                + x[i3               ][j3               ][max(k3-1, 0)     ];
            
            p[i2+1][j2+1][k2+1] *= -1;
            p[i2+1][j2+1][k2+1] += f[i3][j3][k3];
        }
        
        bufs.b[i][j][k] = p[1][1][1];
        
        nu = max(nu, abs(p[1][1][1]));
        rho += ((double) p[1][1][1]) * ((double) p[1][1][1]);
        
        float z = 
            -6 * p[1][1][1]
               + p[0][1][1]
               + p[2][1][1]
               + p[1][0][1]
               + p[1][2][1]
               + p[1][1][0]
               + p[1][1][2];
        
        sigma += ((double) z) * ((double) p[1][1][1]);
    }
    
    if (nu < nuMax) { return; }
    
    for (int iterations=0;; iterations++) 
    {
        bufs.swap();
        
        float new_nu = 0.;
        double new_rho = 0.;
        double new_sigma = 0.;
        
#pragma omp parallel for reduction(max:new_nu) reduction(+:new_rho) reduction(+:new_sigma)
        for (int i = 1; i < XDIM-1; i++)
        for (int j = 1; j < YDIM-1; j++)
        for (int k = 1; k < ZDIM-1; k++)
        {
            float z = 
                -6 * buf.a[i  ][j  ][k  ]
                   + buf.a[i+1][j  ][k  ]
                   + buf.a[i-1][j  ][k  ]
                   + buf.a[i  ][j+1][k  ]
                   + buf.a[i  ][j-1][k  ]
                   + buf.a[i  ][j  ][k+1]
                   + buf.a[i  ][j  ][k-1];
            new_sigma += ((double) buf.a[i][j][k]) * ((double) z);
            
            float alpha = rho / sigma;
            float r_inter = z * -alpha + buf.a[i][j][k];
            
            new_nu = max(new_nu, abs(r_inter);
            new_rho += ((double) r_inter) * ((double) r_inter);
            
            
        }
    }
    
    //cout << "Done, nu=" << to_string(nu) << ", rho=" << to_string(rho) << ", sigma=" << to_string(sigma) << endl;
    
    
    /*
    // Algorithm : Line 2
    ComputeLaplacian(x, z, 2);
    Saxpy(z, f, r, -1, 2);
    float nu = Norm(r, 2);

    // Algorithm : Line 3
    if (nu < nuMax) return;
        
    // Algorithm : Line 4
    Copy(r, p, 4);
    float rho=InnerProduct(p, r, 4);
        
    // Beginning of loop from Line 5
    for(int k=0;;k++)
    {
        std::cout << "Residual norm (nu) after " << k << " iterations = " << nu << std::endl;

        // Algorithm : Line 6
        ComputeLaplacian(p, z, 6);
        float sigma=InnerProduct(p, z, 6);

        // Algorithm : Line 7
        float alpha=rho/sigma;

        // Algorithm : Line 8
        Saxpy(z, r, r, -alpha, 8);
        nu=Norm(r, 8);

        // Algorithm : Lines 9-12
        if (nu < nuMax || k == kMax) {
            Saxpy(p, x, x, alpha, 10);
            std::cout << "Conjugate Gradients terminated after " << k << " iterations; residual norm (nu) = " << nu << std::endl;
            if (writeIterations) WriteAsImage("x", x, k, 0, 127);
            return;
        }
            
        // Algorithm : Line 13
        Copy(r, z, 13);
        float rho_new = InnerProduct(z, r, 13);

        // Algorithm : Line 14
        float beta = rho_new/rho;

        // Algorithm : Line 15
        rho=rho_new;

        // Algorithm : Line 16
        Saxpy(p, x, x, alpha, 16, "1st ");
        Saxpy(p, r, p, beta, 16, "2nd ");

        if (writeIterations) WriteAsImage("x", x, k, 0, 127);
    }
    */
}
