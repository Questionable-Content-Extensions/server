using AutoMapper;
using FluentValidation.AspNetCore;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.IdentityModel.Tokens;
using System;
using System.IO;
using System.Net;
using System.Text;
using QCExtensions.Server.Extensions;
using QCExtensions.Server.Models;
using Pomelo.EntityFrameworkCore.MySql.Infrastructure;

namespace QCExtensions.Server
{
	public class Startup
	{
		private readonly IHostingEnvironment _env;
		private readonly IConfiguration _configuration;

		public Startup(IHostingEnvironment env, IConfiguration configuration)
		{
			_env = env;
			_configuration = configuration;
		}

		public void ConfigureServices(IServiceCollection services)
		{
			// Add Entity Framework services.
			services.AddDbContextPool<ApplicationDbContext>(
                options => options.UseMySql(_configuration.GetConnectionString("Default"),
                    mysqlOptions =>
                    {
                        mysqlOptions.ServerVersion(
							new Version(10, 0, 36),
							ServerType.MariaDb);
                    }
            ));

			services.AddAuthorization();

			services
				.AddAutoMapper()
				.AddMvc()
				.AddFluentValidation(fv => fv.RegisterValidatorsFromAssemblyContaining<ApplicationDbContext>());
		}

		public void Configure(IApplicationBuilder app, IHostingEnvironment env)
		{
			if (env.IsDevelopment())
			{
				app.UseDeveloperExceptionPage();
			}

			app.UseExceptionHandler(
				builder =>
				{
					builder.Run(
						async context =>
						{
							context.Response.StatusCode = (int)HttpStatusCode.InternalServerError;
							context.Response.Headers.Add("Access-Control-Allow-Origin", "*");

							var error = context.Features.Get<IExceptionHandlerFeature>();
							if (error != null)
							{
								context.Response.AddApplicationError(error.Error.Message);
								await context.Response.WriteAsync(error.Error.Message).ConfigureAwait(false);
							}
						}
					);
				}
			);

			app
				.UseDefaultFiles()
				.UseStaticFiles()
				.UseCors(builder =>
					builder.AllowAnyOrigin()
						.AllowAnyMethod()
						.AllowAnyHeader()
						.AllowCredentials()
				)
				.UseMvc(routes => {
					routes.MapRoute("default", "{controller=Home}/{action=Index}/{id?}");

					routes.MapRoute("Spa", "{*url}", defaults: new { controller = "Home", action = "Spa" });
				});
		}
	}
}
