using AutoMapper;
using FluentValidation.AspNetCore;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Net;
using QCExtensions.Server.Extensions;
using Pomelo.EntityFrameworkCore.MySql.Infrastructure;
using QCExtensions.Server.Infrastructure.Services;
using Microsoft.Extensions.Hosting;
using QCExtensions.Server.Infrastructure.Services.Hosted;
using Microsoft.EntityFrameworkCore.Metadata.Internal;
using QCExtensions.Server.Infrastructure.EntityMaterializerSource;
using QCExtensions.Server.Models;

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
			services.AddScoped<ITokenHandler, TokenHandler>();
			services.AddScoped<IActionLogger, ActionLogger>();

			services.AddSingleton<INewsUpdater, NewsUpdater>();
			services.AddSingleton<IHostedService, BackgroundNewsUpdatingService>();

			// Add Entity Framework services.
			services.AddDbContextPool<ApplicationDbContext>(
				options => options.UseMySql(_configuration.GetConnectionString("Default"),
					mysqlOptions =>
					{
						mysqlOptions.ServerVersion(
							new Version(10, 0, 36),
							ServerType.MariaDb);
					}
				).ReplaceService<IEntityMaterializerSource, DateTimeKindEntityMaterializerSource>()
			);

			services.AddAuthorization();

			services
				.AddAutoMapper(opts =>
				{
					opts.CreateMissingTypeMaps = true;
				})
				.AddMemoryCache()
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
				.UseMvc(routes =>
				{
					routes.MapRoute("default", "{controller=Home}/{action=Index}/{id?}");

					routes.MapRoute("Spa", "{*url}", defaults: new { controller = "Home", action = "Spa" });
				});
		}
	}
}
