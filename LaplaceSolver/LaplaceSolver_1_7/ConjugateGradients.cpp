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
    float (&x)[XDIM][YDIM][ZDIM],
    float (&f)[XDIM][YDIM][ZDIM],
    bool writeIterations)
{
    DoubleBuffer bufs;
    
    float nu = 0.;
    
    double rho = 0.;
    double sigma = 0.;
            
#pragma omp parallel for reduction(max:nu) reduction(+:rho) reduction(+:sigma)
    for (int i = 1; i < XDIM-1; i++)
    for (int j = 1; j < YDIM-1; j++)
    for (int k = 1; k < ZDIM-1; k++)
    {
        float z[3][3][3];
        float r[3][3][3];
        
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
            
            // l18
            z[i2+1][j2+1][k2+1] = 
                -6 * x[i3][j3][k3]
                + x[min(i3+1, XDIM-1)][j3               ][k3               ]
                + x[max(i3-1, 0)     ][j3               ][k3               ]
                + x[i3               ][min(j3+1, YDIM-1)][k3               ]
                + x[i3               ][max(j3-1, 0     )][k3               ]
                + x[i3               ][j3               ][min(k3+1, ZDIM-1)]
                + x[i3               ][j3               ][max(k3-1, 0)     ];
            
            r[i2+1][k2+1][j2+1] = z[i2+1][j2+1][k2+1] * -1 + f[max(min(i3, XDIM-1), 0)][max(min(j3, YDIM-1), 0)][max(min(k3, ZDIM-1), 0)];
            
            /*
            // l19
            r = []
            p[i2+1][j2+1][k2+1] *= -1;
            p[i2+1][j2+1][k2+1] += f[i3][j3][k3];
            */
        }
        
        //float r = z[1][1][1] * -1 + f[i][j][k];
        
        nu = max(nu, abs(r[1][1][1]));
        
        // l18 -> buf
        bufs.b[i][j][k] = z[1][1][1];
        rho += ((double) r[1][1][1]) * ((double) r[1][1][1]);
        
        float z2 = 
            -6 * r[1][1][1]
               + r[0][1][1]
               + r[2][1][1]
               + r[1][0][1]
               + r[1][2][1]
               + r[1][1][0]
               + r[1][1][2];
        
        sigma += ((double) r[1][1][1]) * ((double) z2);
        /*
        // l20
        nu = max(nu, abs(p[1][1][1]));
        
        // l27
        rho += ((double) p[1][1][1]) * ((double) p[1][1][1]);
        
        // l35, pre-loop
        float z = 
            -6 * p[1][1][1]
               + p[0][1][1]
               + p[2][1][1]
               + p[1][0][1]
               + p[1][2][1]
               + p[1][1][0]
               + p[1][1][2];
        
        // l36, pre-loop
        sigma += ((double) z) * ((double) p[1][1][1]);
        */
    }
    
    // l23 CTRL
    if (nu < nuMax) { return; }
    
    for (int iterations=0;; iterations++) 
    {
        bufs.swap();
        
        // l39 post-loop
        float alpha = rho / sigma;

        cout << "alpha=" << to_string(alpha) << endl;
        cout << "rho=" << to_string(rho) << endl;
        cout << "sigma=" << to_string(sigma) << endl;
    
        nu = 0.;
        double rho_new = 0.;
    
#pragma omp parallel for reduction(max:nu) reduction(+:rho_new)
        for (int i = 1; i < XDIM-1; i++)
        for (int j = 1; j < YDIM-1; j++)
        for (int k = 1; k < ZDIM-1; k++)
        {
            // l35, pre/post-loop (redundant CPU)
            float z = 
                -6 * bufs.a[i  ][j  ][k  ]
                   + bufs.a[i+1][j  ][k  ]
                   + bufs.a[i-1][j  ][k  ]
                   + bufs.a[i  ][j+1][k  ]
                   + bufs.a[i  ][j-1][k  ]
                   + bufs.a[i  ][j  ][k+1]
                   + bufs.a[i  ][j  ][k-1];
            
            // l42, pre/post-loop (redundant CPU)
            float m = z * -alpha + m;
            // l43, post-loop
            nu = max(nu, abs(m));
            // l55, post-loop
            rho_new += ((double) m) * ((double) m);
        }
        
        // l46 CTRL
        if (nu < nuMax || iterations == kMax) {
#pragma omp parallel for
            for (int i = 1; i < XDIM-1; i++)
            for (int j = 1; j < YDIM-1; j++)
            for (int k = 1; k < ZDIM-1; k++)
            {
                x[i][j][k] = bufs.a[i][j][k] * alpha + x[i][j][k];
            }
            std::cout << "Conjugate Gradients terminated after " << iterations << " iterations; residual norm (nu) = " << nu << std::endl;
            if (writeIterations) WriteAsImage("x", x, iterations, 0, 127);
            return;
        }
        
        std::cout << "Residual norm (nu) after " << iterations << " iterations = " << nu << std::endl;
        
        // l58 post-loop
        float beta = rho_new / rho;
        
        
        // reponsibilities of pre-loop
        //  - initialize rho and sigma
        //  - write to p
        
        sigma = 0.;
#pragma omp parallel for reduction(+:sigma)
        for (int i = 1; i < XDIM-1; i++)
        for (int j = 1; j < YDIM-1; j++)
        for (int k = 1; k < ZDIM-1; k++)
        {
            // l35, pre/post-loop (redundant CPU)
            float z = 
                -6 * bufs.a[i  ][j  ][k  ]
                   + bufs.a[i+1][j  ][k  ]
                   + bufs.a[i-1][j  ][k  ]
                   + bufs.a[i  ][j+1][k  ]
                   + bufs.a[i  ][j-1][k  ]
                   + bufs.a[i  ][j  ][k+1]
                   + bufs.a[i  ][j  ][k-1];
            
            // l42, pre/post-loop (redundant CPU)
            float m = z * -alpha + m;
            
            // BUT these lines are actually part of the post-loop
            // l64
            x[i][j][k] = bufs.a[i][j][k] * alpha + x[i][j][k];
            // l65
            bufs.b[i][j][k] = bufs.a[i][j][k] * beta + m;
            
            sigma += ((double) bufs.a[i][j][k]) * ((double) z);
        }
        
        // and THIS is also part of the post-loop actually
        if (writeIterations) { WriteAsImage("x", x, iterations, 0, 127); }
        
        rho = rho_new;    
    }
}