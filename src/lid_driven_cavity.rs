mod correct_parameters;
mod face_velocity;
mod get_links_momentum;
mod get_links_pressure_correction;
mod postprocessing;
mod residuals;
mod solver;
mod solver_correction;

use itertools_num::linspace;

use crate::Case;

use self::{face_velocity::Faces, residuals::Residuals};

pub struct LidDrivenCavity {
    pub nx: usize,
    pub ny: usize,
    pub re: f32, //Reynolds number
    pub dx: f32,
    pub dy: f32,
    pub nu: f32,
    pub rho: f32,
    pub relax_uv: f32,
    pub relax_p: f32,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub links: Vec<Links>,
    pub plinks: Vec<Links>,
    pub source_x: Vec<f32>,
    pub source_y: Vec<f32>,
    pub source_p: Vec<f32>,
    pub a_0: Vec<f32>,
    pub a_p0: Vec<f32>,
    pub faces: Vec<Faces>,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub p: Vec<f32>,
    pub pc: Vec<f32>,
    pub residuals: Residuals,
}

impl LidDrivenCavity {
    pub fn new(nx: usize, ny: usize, re: f32) -> Self {
        let dx = 1.0 / (nx as f32);
        let dy = 1.0 / (ny as f32);
        let x = linspace::<f32>(0.0, 1.0, nx).collect();
        let y = linspace::<f32>(0.0, 1.0, ny).collect();
        let u = vec![0.0; nx * ny];

        let v = vec![0.0; nx * ny];
        let p = vec![0.0; nx * ny];
        let pc = vec![0.0; nx * ny];
        let links = vec![Links::default(); nx * ny];
        let plinks = links.clone();
        let source_x = vec![0.0; nx * ny];
        let source_y = source_x.clone();
        let source_p = source_x.clone();
        let a_0 = vec![0.0; nx * ny];
        let a_p0 = a_0.clone();
        let faces = vec![Faces::default(); ny * nx];
        let residuals = Residuals::default();

        let relax_uv = 0.8;
        let relax_p = 0.1;

        Self {
            nx,
            ny,
            re,
            nu: 1.0 / re,
            dx,
            dy,
            rho: 1.0,
            u,
            v,
            relax_uv,
            relax_p,
            x,
            y,
            links,
            plinks,
            source_x,
            source_y,
            source_p,
            a_0,
            a_p0,
            faces,
            p,
            pc,
            residuals,
        }
    }
}
impl Case for LidDrivenCavity {
    fn iterate(&mut self) -> bool {
        self.get_links_momentum();

        self.save_u_residual();
        let mut u = std::mem::take(&mut self.u);
        self.solver_correction(&mut u, &self.a_0, &self.links, &self.source_x, 4, 0.2);
        self.u = u;

        self.save_v_residual();
        let mut v = std::mem::take(&mut self.v);
        self.solver_correction(&mut v, &self.a_0, &self.links, &self.source_y, 4, 0.2);
        self.v = v;

        self.get_face_velocities();
        self.get_links_pressure_correction();

        self.save_pressure_residual();
        self.pc = vec![0.0; self.nx * self.ny];
        let mut pc = std::mem::take(&mut self.pc);
        self.solver(&mut pc, &self.a_p0, &self.plinks, &self.source_p, 20);
        self.pc = pc;
        dbg!(self.pc.iter().fold(0.0, |acc, x| acc + x.abs()));

        self.correct_cell_velocities();
        self.correct_face_velocities();
        self.correct_pressure();

        !self.u.iter().fold(0.0, |acc, x| acc + x.abs()).is_nan()
    }

    fn postprocessing(&self, plot: &mut crate::plotter::Plot<'_>, iter: usize) {
        self.plot(plot, iter);
    }
}

#[derive(Clone, Default)]
pub struct Links {
    pub a_e: f32,
    pub a_w: f32,
    pub a_n: f32,
    pub a_s: f32,
}

impl Links {
    pub fn set_links(&mut self, a_e: f32, a_w: f32, a_n: f32, a_s: f32) {
        self.a_e = a_e;
        self.a_w = a_w;
        self.a_n = a_n;
        self.a_s = a_s;
    }
}
